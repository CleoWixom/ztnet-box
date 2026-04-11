//! Bridge setup and teardown via `ip link` + systemd-networkd unit files.
//!
//! Approach (from https://docs.zerotier.com/bridging/):
//!   1. `ip link add br0 type bridge`
//!   2. `ip link set <phy> master br0`
//!   3. `ip link set <zt>  master br0`
//!   4. `ip link set br0 up`
//!   5. Optionally: `ip addr add <addr> dev br0`
//!   6. Write systemd-networkd .netdev + .network for persistence

use super::BridgeConfig;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Command failed: {0}")]
    Command(String),
    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),
}

pub fn apply(cfg: &BridgeConfig) -> Result<(), BridgeError> {
    #[cfg(not(target_os = "linux"))]
    return Err(BridgeError::UnsupportedPlatform(
        "Layer 2 Bridge requires Linux".into(),
    ));

    #[cfg(target_os = "linux")]
    {
        let br = &cfg.bridge_iface;

        // 1. Create bridge (ignore error if already exists)
        let _ = ip(&["link", "add", br, "type", "bridge"]);

        // 2. Attach physical interface
        ip(&["link", "set", &cfg.phy_iface, "master", br])?;

        // 3. Attach ZeroTier interface
        ip(&["link", "set", &cfg.zt_iface, "master", br])?;

        // 4. Bring bridge up
        ip(&["link", "set", br, "up"])?;

        // 5. Assign static address if requested
        if let Some(ref addr) = cfg.bridge_addr {
            // Flush existing addresses first to avoid duplicates
            let _ = ip(&["addr", "flush", "dev", br]);
            ip(&["addr", "add", addr, "dev", br])?;
        }

        // 6. Add default gateway if requested
        if let Some(ref gw) = cfg.gateway {
            let _ = ip(&["route", "del", "default"]);
            ip(&["route", "add", "default", "via", gw])?;
        }

        // 7. Write systemd-networkd unit files for persistence
        write_netdev(cfg)?;
        write_network(cfg)?;

        // 8. Reload systemd-networkd
        let _ = std::process::Command::new("networkctl")
            .arg("reload")
            .status();

        tracing::info!(
            zt = %cfg.zt_iface,
            phy = %cfg.phy_iface,
            br = %cfg.bridge_iface,
            "Layer 2 Bridge enabled"
        );
        Ok(())
    }
}

pub fn remove(cfg: &BridgeConfig) -> Result<(), BridgeError> {
    #[cfg(not(target_os = "linux"))]
    return Err(BridgeError::UnsupportedPlatform(
        "Layer 2 Bridge requires Linux".into(),
    ));

    #[cfg(target_os = "linux")]
    {
        let br = &cfg.bridge_iface;

        // Detach members and bring bridge down
        let _ = ip(&["link", "set", &cfg.zt_iface, "nomaster"]);
        let _ = ip(&["link", "set", &cfg.phy_iface, "nomaster"]);
        let _ = ip(&["link", "set", br, "down"]);
        let _ = ip(&["link", "del", br]);

        // Remove systemd-networkd unit files
        remove_netdev_files(cfg);

        let _ = std::process::Command::new("networkctl")
            .arg("reload")
            .status();

        tracing::info!(
            zt = %cfg.zt_iface,
            phy = %cfg.phy_iface,
            br = %cfg.bridge_iface,
            "Layer 2 Bridge disabled"
        );
        Ok(())
    }
}

// ── systemd-networkd unit writers ─────────────────────────────────────────────

#[cfg(target_os = "linux")]
fn write_netdev(cfg: &BridgeConfig) -> Result<(), BridgeError> {
    let path = format!("/etc/systemd/network/10-ztnet-{}.netdev", cfg.bridge_iface);
    let content = format!(
        "[NetDev]\nName={br}\nKind=bridge\n",
        br = cfg.bridge_iface
    );
    std::fs::write(&path, content)?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn write_network(cfg: &BridgeConfig) -> Result<(), BridgeError> {
    // Bridge master network file
    let master_path = format!(
        "/etc/systemd/network/10-ztnet-{}-master.network",
        cfg.bridge_iface
    );
    let mut master = format!(
        "[Match]\nName={zt}\nName={phy}\n\n[Network]\nBridge={br}\n",
        zt = cfg.zt_iface,
        phy = cfg.phy_iface,
        br = cfg.bridge_iface,
    );
    // Disable DHCP on enslaved interfaces
    master.push_str("DHCP=no\n");
    std::fs::write(&master_path, master)?;

    // Bridge itself network file
    let br_path = format!(
        "/etc/systemd/network/10-ztnet-{}.network",
        cfg.bridge_iface
    );
    let mut br_net = format!("[Match]\nName={br}\n\n[Network]\n", br = cfg.bridge_iface);
    match &cfg.bridge_addr {
        Some(addr) => {
            br_net.push_str("DHCP=no\n");
            br_net.push_str(&format!("\n[Address]\nAddress={addr}\n"));
            if let Some(ref gw) = cfg.gateway {
                br_net.push_str(&format!("\n[Route]\nGateway={gw}\n"));
            }
        }
        None => {
            br_net.push_str("DHCP=yes\n");
        }
    }
    std::fs::write(&br_path, br_net)?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn remove_netdev_files(cfg: &BridgeConfig) {
    let files = [
        format!(
            "/etc/systemd/network/10-ztnet-{}.netdev",
            cfg.bridge_iface
        ),
        format!(
            "/etc/systemd/network/10-ztnet-{}-master.network",
            cfg.bridge_iface
        ),
        format!(
            "/etc/systemd/network/10-ztnet-{}.network",
            cfg.bridge_iface
        ),
    ];
    for f in &files {
        let _ = std::fs::remove_file(f);
    }
}

// ── ip helper ─────────────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
fn ip(args: &[&str]) -> Result<(), BridgeError> {
    let status = std::process::Command::new("ip")
        .args(args)
        .status()
        .map_err(|e| BridgeError::Command(format!("ip spawn: {e}")))?;
    if status.success() {
        Ok(())
    } else {
        Err(BridgeError::Command(format!(
            "ip {:?} exited with {status}",
            args
        )))
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> BridgeConfig {
        BridgeConfig {
            zt_iface: "zt3abc".into(),
            phy_iface: "eth0".into(),
            bridge_iface: "br0".into(),
            bridge_addr: Some("192.168.1.10/24".into()),
            gateway: Some("192.168.1.1".into()),
            network_id: "a1b2c3d4e5f6a7b8".into(),
        }
    }

    #[test]
    fn bridge_config_roundtrip() {
        let c = cfg();
        let json = serde_json::to_string(&c).unwrap();
        let parsed: BridgeConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.zt_iface, "zt3abc");
        assert_eq!(parsed.bridge_iface, "br0");
        assert_eq!(parsed.bridge_addr.as_deref(), Some("192.168.1.10/24"));
    }

    #[test]
    fn bridge_config_no_addr() {
        let c = BridgeConfig {
            zt_iface: "zt0".into(),
            phy_iface: "eth1".into(),
            bridge_iface: "br0".into(),
            bridge_addr: None,
            gateway: None,
            network_id: "a1b2c3d4e5f6a7b8".into(),
        };
        assert!(c.bridge_addr.is_none());
        assert!(c.gateway.is_none());
    }

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn apply_returns_unsupported_on_non_linux() {
        let err = apply(&cfg()).unwrap_err();
        assert!(err.to_string().contains("Linux"));
    }

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn remove_returns_unsupported_on_non_linux() {
        let err = remove(&cfg()).unwrap_err();
        assert!(err.to_string().contains("Linux"));
    }
}
