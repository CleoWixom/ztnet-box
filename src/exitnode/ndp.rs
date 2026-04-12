//! NDP Proxy support via `ndppd` (Neighbor Discovery Protocol Proxy Daemon).
//!
//! Provides native IPv6 delegation for Exit Node without NAT:
//!   - On the gateway: ndppd answers NDP requests on the WAN interface,
//!     forwarding them for addresses assigned to ZeroTier clients.
//!   - This allows clients with real IPv6 addresses to be reachable from the
//!     internet without any ip6tables MASQUERADE.
//!
//! References:
//!   - https://docs.zerotier.com/exitnode/
//!   - https://github.com/DanielAdolfsson/ndppd

use serde::Serialize;
use which::which;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct NdpStatus {
    /// ndppd binary found on PATH
    pub available: bool,
    /// ndppd is currently running (systemd active or process detected)
    pub running: bool,
    /// Config file exists at /etc/ndppd.conf
    pub config_exists: bool,
    /// systemd unit enabled
    pub enabled: bool,
    /// Path to ndppd binary if found
    pub binary_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NdpConfig {
    /// WAN interface that receives NDP queries (e.g. "eth0")
    pub wan_iface: String,
    /// ZeroTier interface prefix to proxy NDP for (e.g. "zt")
    pub zt_prefix: String,
    /// IPv6 prefix delegated to ZT clients (e.g. "2001:db8::/64")
    pub ipv6_prefix: String,
}

// ── Status ────────────────────────────────────────────────────────────────────

pub fn check_status() -> NdpStatus {
    let binary = which("ndppd").ok();
    let available = binary.is_some();

    let running = is_ndppd_running();
    let config_exists = std::path::Path::new("/etc/ndppd.conf").exists();
    let enabled = is_ndppd_enabled();

    NdpStatus {
        available,
        running,
        config_exists,
        enabled,
        binary_path: binary.map(|p| p.display().to_string()),
    }
}

fn is_ndppd_running() -> bool {
    // Try systemctl first
    if let Ok(out) = std::process::Command::new("systemctl")
        .args(["is-active", "ndppd"])
        .output()
    {
        if out.status.success() {
            return true;
        }
    }
    // Fallback: check process list
    if let Ok(out) = std::process::Command::new("pgrep").arg("ndppd").output() {
        return out.status.success();
    }
    false
}

fn is_ndppd_enabled() -> bool {
    std::process::Command::new("systemctl")
        .args(["is-enabled", "ndppd"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// ── Install ───────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum NdpError {
    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),
    #[error("Command failed: {0}")]
    Command(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub fn install() -> Result<NdpStatus, NdpError> {
    #[cfg(not(target_os = "linux"))]
    return Err(NdpError::UnsupportedPlatform(
        "ndppd requires Linux".into(),
    ));

    #[cfg(target_os = "linux")]
    {
        let pm = detect_pm().ok_or_else(|| {
            NdpError::Command("No supported package manager (apt/dnf/pacman) found".into())
        })?;

        let ok = match pm {
            Pm::Apt => run(&["apt-get", "install", "-y", "ndppd"]),
            Pm::Dnf => run(&["dnf", "install", "-y", "ndppd"]),
            Pm::Pacman => run(&["pacman", "-S", "--noconfirm", "ndppd"]),
        };

        if !ok {
            return Err(NdpError::Command("Failed to install ndppd".into()));
        }

        tracing::info!("ndppd installed");
        Ok(check_status())
    }
}

// ── Configure + Enable ────────────────────────────────────────────────────────

/// Write /etc/ndppd.conf and enable+start ndppd via systemd.
pub fn enable(cfg: &NdpConfig) -> Result<NdpStatus, NdpError> {
    #[cfg(not(target_os = "linux"))]
    return Err(NdpError::UnsupportedPlatform(
        "ndppd requires Linux".into(),
    ));

    #[cfg(target_os = "linux")]
    {
        write_config(cfg)?;
        reload_and_start()?;
        tracing::info!(
            wan = %cfg.wan_iface,
            prefix = %cfg.ipv6_prefix,
            "ndppd enabled"
        );
        Ok(check_status())
    }
}

/// Stop and disable ndppd; optionally remove /etc/ndppd.conf.
pub fn disable(remove_config: bool) -> Result<NdpStatus, NdpError> {
    #[cfg(not(target_os = "linux"))]
    return Err(NdpError::UnsupportedPlatform(
        "ndppd requires Linux".into(),
    ));

    #[cfg(target_os = "linux")]
    {
        let _ = run_cmd("systemctl", &["stop", "ndppd"]);
        let _ = run_cmd("systemctl", &["disable", "ndppd"]);
        if remove_config {
            let _ = std::fs::remove_file("/etc/ndppd.conf");
        }
        tracing::info!("ndppd disabled");
        Ok(check_status())
    }
}

// ── Config writer ─────────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
fn write_config(cfg: &NdpConfig) -> Result<(), NdpError> {
    // ndppd.conf format:
    //   proxy <WAN_IFACE> {
    //       rule <PREFIX> {
    //           iface <ZT_PREFIX>+
    //       }
    //   }
    let content = format!(
        "# Generated by ztnet-box\n\
         proxy {wan} {{\n\
         \trule {prefix} {{\n\
         \t\tiface {zt}+\n\
         \t}}\n\
         }}\n",
        wan = cfg.wan_iface,
        prefix = cfg.ipv6_prefix,
        zt = cfg.zt_prefix,
    );
    std::fs::write("/etc/ndppd.conf", content)?;
    tracing::info!(path = "/etc/ndppd.conf", "ndppd config written");
    Ok(())
}

#[cfg(target_os = "linux")]
fn reload_and_start() -> Result<(), NdpError> {
    let _ = run_cmd("systemctl", &["daemon-reload"]);
    if !run_cmd("systemctl", &["enable", "--now", "ndppd"]) {
        return Err(NdpError::Command(
            "systemctl enable --now ndppd failed".into(),
        ));
    }
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
enum Pm {
    Apt,
    Dnf,
    Pacman,
}

#[cfg(target_os = "linux")]
fn detect_pm() -> Option<Pm> {
    if which("apt-get").is_ok() {
        return Some(Pm::Apt);
    }
    if which("dnf").is_ok() {
        return Some(Pm::Dnf);
    }
    if which("pacman").is_ok() {
        return Some(Pm::Pacman);
    }
    None
}

#[cfg(target_os = "linux")]
fn run(args: &[&str]) -> bool {
    let Some((cmd, rest)) = args.split_first() else {
        return false;
    };
    std::process::Command::new(cmd)
        .args(rest)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(target_os = "linux")]
fn run_cmd(cmd: &str, args: &[&str]) -> bool {
    std::process::Command::new(cmd)
        .args(args)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_status_does_not_panic() {
        let s = check_status();
        let _ = s.running;
        let _ = s.config_exists;
    }

    #[test]
    fn ndp_config_fields() {
        let c = NdpConfig {
            wan_iface: "eth0".into(),
            zt_prefix: "zt".into(),
            ipv6_prefix: "2001:db8::/64".into(),
        };
        assert_eq!(c.wan_iface, "eth0");
        assert_eq!(c.zt_prefix, "zt");
    }

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn install_returns_unsupported_on_non_linux() {
        let err = install().unwrap_err();
        assert!(err.to_string().contains("Linux"));
    }

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn enable_returns_unsupported_on_non_linux() {
        let cfg = NdpConfig {
            wan_iface: "eth0".into(),
            zt_prefix: "zt".into(),
            ipv6_prefix: "2001:db8::/64".into(),
        };
        let err = enable(&cfg).unwrap_err();
        assert!(err.to_string().contains("Linux"));
    }
}
