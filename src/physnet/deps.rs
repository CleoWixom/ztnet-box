//! Dependency checking for Physical Network Routing.

use serde::Serialize;
use which::which;

#[derive(Debug, Clone, Serialize)]
pub struct PhysNetDeps {
    pub iptables: Option<std::path::PathBuf>,
    pub is_root: bool,
    pub ip_forward_enabled: bool,
    pub missing: Vec<String>,
}

pub fn check() -> PhysNetDeps {
    let iptables = which("iptables").ok();
    let is_root = is_root();
    let ip_forward = std::fs::read_to_string("/proc/sys/net/ipv4/ip_forward")
        .map(|s| s.trim() == "1")
        .unwrap_or(false);

    let mut missing = Vec::new();
    if iptables.is_none() {
        missing.push("iptables".into());
    }
    if !is_root {
        missing.push("root access required".into());
    }

    PhysNetDeps {
        iptables,
        is_root,
        ip_forward_enabled: ip_forward,
        missing,
    }
}

#[cfg(unix)]
fn is_root() -> bool {
    nix::unistd::getuid().is_root()
}
#[cfg(not(unix))]
fn is_root() -> bool {
    false
}
