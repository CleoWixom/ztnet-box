use serde::Serialize;
use which::which;

#[derive(Debug, Clone, Serialize)]
pub struct BridgeDeps {
    /// systemd-networkd available and (on Linux) active
    pub systemd_networkd: bool,
    /// systemd-resolved present (informational)
    pub systemd_resolved: bool,
    /// Running as root
    pub is_root: bool,
    /// `ip` command available (iproute2)
    pub iproute2: bool,
    /// dhcpcd is installed — conflicts with systemd-networkd
    pub dhcpcd_conflict: bool,
    /// ifupdown is installed — conflicts with systemd-networkd
    pub ifupdown_conflict: bool,
    /// Packages that must be removed / installed before bridging
    pub missing: Vec<String>,
}

pub fn check() -> BridgeDeps {
    let iproute2 = which("ip").is_ok();
    let systemd_networkd = which("networkctl").is_ok();
    let systemd_resolved = which("resolvectl").is_ok();
    let is_root = is_root();
    let dhcpcd_conflict = which("dhcpcd").is_ok();
    let ifupdown_conflict = which("ifup").is_ok();

    let mut missing = Vec::new();
    if !iproute2 {
        missing.push("iproute2 (ip command not found)".into());
    }
    if !systemd_networkd {
        missing.push("systemd-networkd (networkctl not found)".into());
    }
    if dhcpcd_conflict {
        missing.push("remove dhcpcd5 (conflicts with systemd-networkd)".into());
    }
    if ifupdown_conflict {
        missing.push("remove ifupdown (conflicts with systemd-networkd)".into());
    }

    BridgeDeps {
        systemd_networkd,
        systemd_resolved,
        is_root,
        iproute2,
        dhcpcd_conflict,
        ifupdown_conflict,
        missing,
    }
}

pub fn install(_prefer_remove_conflicts: bool) -> Result<BridgeDeps, String> {
    #[cfg(not(target_os = "linux"))]
    return Err("Bridge deps install requires Linux".into());

    #[cfg(target_os = "linux")]
    {
        if _prefer_remove_conflicts {
            // Remove conflicting packages
            for pkg in &["dhcpcd5", "ifupdown", "isc-dhcp-client"] {
                let _ = std::process::Command::new("apt-get")
                    .args(["remove", "-y", pkg])
                    .status();
            }
        }
        // Ensure systemd-networkd is enabled
        let _ = std::process::Command::new("systemctl")
            .args(["enable", "--now", "systemd-networkd"])
            .status();
        Ok(check())
    }
}

fn is_root() -> bool {
    #[cfg(unix)]
    return nix::unistd::getuid().is_root();
    #[cfg(not(unix))]
    return false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_does_not_panic() {
        let d = check();
        let _ = d.is_root;
        let _ = d.systemd_networkd;
    }

    #[test]
    fn missing_populated_on_no_ip() {
        // On CI iproute2 is present — just verify structure
        let d = check();
        if !d.iproute2 {
            assert!(d.missing.iter().any(|m| m.contains("iproute2")));
        }
    }
}
