#[derive(Debug, serde::Serialize)]
pub struct DepsCheck {
    pub nft: bool,
    pub iptables: bool,
    pub ip_forward_enabled: bool,
}

pub fn check() -> DepsCheck {
    let nft = which::which("nft").is_ok();
    let iptables = which::which("iptables").is_ok();
    let ip_forward = std::fs::read_to_string("/proc/sys/net/ipv4/ip_forward")
        .map(|s| s.trim() == "1")
        .unwrap_or(false);
    DepsCheck {
        nft,
        iptables,
        ip_forward_enabled: ip_forward,
    }
}
