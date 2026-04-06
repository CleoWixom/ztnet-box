#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum FirewallBackend {
    Nftables,
    Iptables,
    Unsupported,
}

pub fn detect_backend(prefer_nftables: bool) -> FirewallBackend {
    if prefer_nftables && which::which("nft").is_ok() {
        return FirewallBackend::Nftables;
    }
    if which::which("iptables").is_ok() {
        return FirewallBackend::Iptables;
    }
    if which::which("nft").is_ok() {
        return FirewallBackend::Nftables;
    }
    FirewallBackend::Unsupported
}
