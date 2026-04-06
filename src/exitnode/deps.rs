use serde::Serialize;
use std::path::PathBuf;
use which::which;

#[derive(Debug, Clone, Serialize)]
pub struct DepsStatus {
    pub iptables: Option<PathBuf>,
    pub nftables: Option<PathBuf>,
    pub is_root: bool,
    pub ip_forward_enabled: bool,
    pub missing: Vec<String>,
}

pub fn check_deps() -> DepsStatus {
    let iptables = which("iptables").ok();
    let nftables = which("nft").ok();
    let is_root = is_root();
    let ip_forward = read_ip_forward();

    let mut missing = Vec::new();
    if iptables.is_none() && nftables.is_none() {
        missing.push("iptables or nftables".into());
    }
    if !ip_forward {
        missing.push("ip_forward (will be enabled automatically)".into());
    }

    DepsStatus {
        iptables,
        nftables,
        is_root,
        ip_forward_enabled: ip_forward,
        missing,
    }
}

pub fn install_missing(prefer_nftables: bool) -> Result<DepsStatus, String> {
    let pm =
        detect_package_manager().ok_or_else(|| "No supported package manager found".to_string())?;

    let pkg = if prefer_nftables {
        "nftables"
    } else {
        "iptables"
    };

    let ok = match pm {
        Pm::Apt => run(&["/usr/bin/apt-get", "install", "-y", pkg]),
        Pm::Dnf => run(&["/usr/bin/dnf", "install", "-y", pkg]),
        Pm::Pacman => run(&["/usr/bin/pacman", "-S", "--noconfirm", pkg]),
    };

    if ok {
        Ok(check_deps())
    } else {
        Err(format!("Failed to install {pkg}"))
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

enum Pm {
    Apt,
    Dnf,
    Pacman,
}

fn detect_package_manager() -> Option<Pm> {
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

fn is_root() -> bool {
    // nix::unistd::getuid() == 0, with fallback for non-unix
    #[cfg(unix)]
    {
        nix::unistd::getuid().is_root()
    }
    #[cfg(not(unix))]
    {
        false
    }
}

fn read_ip_forward() -> bool {
    std::fs::read_to_string("/proc/sys/net/ipv4/ip_forward")
        .map(|s| s.trim() == "1")
        .unwrap_or(false)
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_deps_does_not_panic() {
        let d = check_deps();
        // is_root may be true or false depending on environment
        let _ = d.is_root;
    }

    #[test]
    fn missing_empty_when_tools_present() {
        let d = check_deps();
        // On CI with iptables available, missing should not include iptables/nftables
        if d.iptables.is_some() || d.nftables.is_some() {
            assert!(!d.missing.iter().any(|m| m.contains("iptables")));
        }
    }
}
