//! iptables rules for Physical Network Routing.
//! https://docs.zerotier.com/route-between-phys-and-virt/

use super::PhysNetConfig;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PhysNetError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Command failed: {0}")]
    Command(String),
    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),
}

pub fn apply(_cfg: &PhysNetConfig) -> Result<(), PhysNetError> {
    #[cfg(not(target_os = "linux"))]
    return Err(PhysNetError::UnsupportedPlatform(
        "Physical Network Routing requires Linux".into(),
    ));

    #[cfg(target_os = "linux")]
    {
        enable_ip_forward()?;

        // NAT masquerade outbound to physical interface
        run_iptables(&[
            "-t",
            "nat",
            "-A",
            "POSTROUTING",
            "-o",
            &_cfg.phy_iface,
            "-j",
            "MASQUERADE",
        ])?;

        // Allow established/related traffic back from physical to ZT
        run_iptables(&[
            "-A",
            "FORWARD",
            "-i",
            &_cfg.phy_iface,
            "-o",
            &_cfg.zt_iface,
            "-m",
            "state",
            "--state",
            "RELATED,ESTABLISHED",
            "-j",
            "ACCEPT",
        ])?;

        // Allow new traffic from ZT to physical
        run_iptables(&[
            "-A",
            "FORWARD",
            "-i",
            &_cfg.zt_iface,
            "-o",
            &_cfg.phy_iface,
            "-j",
            "ACCEPT",
        ])?;

        // Persist rules
        persist_rules();

        tracing::info!(
            zt = %_cfg.zt_iface,
            phy = %_cfg.phy_iface,
            subnet = %_cfg.phy_subnet,
            "Physical Network Routing enabled"
        );
        Ok(())
    }
}

pub fn remove(_cfg: &PhysNetConfig) -> Result<(), PhysNetError> {
    #[cfg(not(target_os = "linux"))]
    return Err(PhysNetError::UnsupportedPlatform(
        "Physical Network Routing requires Linux".into(),
    ));

    #[cfg(target_os = "linux")]
    {
        let _ = run_iptables(&[
            "-t",
            "nat",
            "-D",
            "POSTROUTING",
            "-o",
            &_cfg.phy_iface,
            "-j",
            "MASQUERADE",
        ]);
        let _ = run_iptables(&[
            "-D",
            "FORWARD",
            "-i",
            &_cfg.phy_iface,
            "-o",
            &_cfg.zt_iface,
            "-m",
            "state",
            "--state",
            "RELATED,ESTABLISHED",
            "-j",
            "ACCEPT",
        ]);
        let _ = run_iptables(&[
            "-D",
            "FORWARD",
            "-i",
            &_cfg.zt_iface,
            "-o",
            &_cfg.phy_iface,
            "-j",
            "ACCEPT",
        ]);

        tracing::info!(
            zt = %_cfg.zt_iface,
            phy = %_cfg.phy_iface,
            "Physical Network Routing disabled"
        );
        Ok(())
    }
}

#[cfg(target_os = "linux")]
fn enable_ip_forward() -> Result<(), PhysNetError> {
    std::fs::write("/proc/sys/net/ipv4/ip_forward", "1\n")?;
    let _ = append_sysctl("net.ipv4.ip_forward", "1");
    Ok(())
}

#[cfg(target_os = "linux")]
fn append_sysctl(key: &str, value: &str) -> Result<(), std::io::Error> {
    let path = "/etc/sysctl.conf";
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let entry = format!("{key} = {value}");
    if !content.contains(key) {
        let append = format!("\n# Added by ztnet-box\n{entry}\n");
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new().append(true).open(path)?;
        f.write_all(append.as_bytes())?;
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn run_iptables(args: &[&str]) -> Result<(), PhysNetError> {
    let status = std::process::Command::new("iptables")
        .args(args)
        .status()
        .map_err(|e| PhysNetError::Command(format!("iptables spawn: {e}")))?;
    if status.success() {
        Ok(())
    } else {
        Err(PhysNetError::Command(format!(
            "iptables {:?} exited: {status}",
            args
        )))
    }
}

#[cfg(target_os = "linux")]
fn persist_rules() {
    if which::which("netfilter-persistent").is_ok() {
        let _ = std::process::Command::new("netfilter-persistent")
            .arg("save")
            .status();
    } else if which::which("iptables-save").is_ok() {
        let _ = std::process::Command::new("sh")
            .args(["-c", "iptables-save > /etc/iptables/rules.v4"])
            .status();
    }
}
