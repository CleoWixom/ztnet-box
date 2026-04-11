use serde::Serialize;
use std::path::PathBuf;
use which::which;

use super::rules::ExitNodeRules;

#[derive(Debug, Clone, Serialize)]
pub struct DepsStatus {
    pub iptables: Option<PathBuf>,
    pub nftables: Option<PathBuf>,
    pub ip6tables: Option<PathBuf>,
    pub is_root: bool,
    pub ip_forward_enabled: bool,
    pub ipv6_forward_enabled: bool,
    /// rp_filter value (0 or 2) is required on the gateway so client traffic
    /// with allowDefault=1 passes the reverse-path filter.
    /// See: https://docs.zerotier.com/exitnode/#a-linux-gotcha-rp_filter
    pub rp_filter_ok: bool,
    /// netfilter-persistent or iptables-save available for rule persistence
    pub persist_available: bool,
    pub missing: Vec<String>,
}

pub fn check_deps() -> DepsStatus {
    let iptables = which("iptables").ok();
    let nftables = which("nft").ok();
    let ip6tables = which("ip6tables").ok();
    let is_root = is_root();
    let ip_forward = read_ip_forward();
    let ipv6_forward = read_ipv6_forward();
    let rp_filter_ok = ExitNodeRules::check_rp_filter();
    let persist_available = which("netfilter-persistent").is_ok()
        || which("iptables-save").is_ok()
        || which("nft").is_ok();

    let mut missing = Vec::new();
    if iptables.is_none() && nftables.is_none() {
        missing.push("iptables or nftables".into());
    }
    if !ip_forward {
        missing.push("ip_forward (will be enabled automatically)".into());
    }
    if !rp_filter_ok {
        missing
            .push("rp_filter=2 (required for client traffic; will be fixed automatically)".into());
    }

    DepsStatus {
        iptables,
        nftables,
        ip6tables,
        is_root,
        ip_forward_enabled: ip_forward,
        ipv6_forward_enabled: ipv6_forward,
        rp_filter_ok,
        persist_available,
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

fn read_ipv6_forward() -> bool {
    std::fs::read_to_string("/proc/sys/net/ipv6/conf/all/forwarding")
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
        let _ = d.is_root;
        let _ = d.rp_filter_ok;
        let _ = d.persist_available;
    }

    #[test]
    fn missing_empty_when_tools_present() {
        let d = check_deps();
        if d.iptables.is_some() || d.nftables.is_some() {
            assert!(!d.missing.iter().any(|m| m.contains("iptables")));
        }
    }
}
