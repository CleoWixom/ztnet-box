//! Physical Network Routing — NAT/Masquerade between ZeroTier and physical LAN.
//! Docs: https://docs.zerotier.com/route-between-phys-and-virt/
//!
//! Enables remote access to a physical LAN via ZeroTier without requiring
//! access to the physical router. Uses iptables masquerade on a Linux PC/Pi.

pub mod conflicts;
pub mod deps;
pub mod rules;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ── Config & State ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysNetConfig {
    /// ZeroTier interface name (e.g. "zt7nnig26")
    pub zt_iface: String,
    /// Physical interface to the LAN (e.g. "eth0")
    pub phy_iface: String,
    /// Physical subnet in CIDR (e.g. "192.168.100.0/24")
    /// The managed route in ZeroTier Central should use /23 (one size larger)
    pub phy_subnet: String,
    /// ZeroTier IP address of this node on the ZT network (the gateway address)
    pub zt_addr: String,
    /// ZeroTier network ID
    pub network_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PhysNetState {
    pub enabled: bool,
    pub config: Option<PhysNetConfig>,
    pub applied_at: Option<DateTime<Utc>>,
}

/// Hint shown in UI for configuring the ZeroTier Central managed route.
/// The route should use a slightly larger subnet than the physical one
/// so that nodes on both networks prefer the physical connection (longest-prefix-match).
pub fn managed_route_hint(phy_subnet: &str) -> String {
    widen_subnet(phy_subnet).unwrap_or_else(|| phy_subnet.to_string())
}

/// Widen a CIDR by 1 prefix bit (e.g. /24 → /23) for longest-prefix-match trick.
fn widen_subnet(cidr: &str) -> Option<String> {
    let parts: Vec<&str> = cidr.splitn(2, '/').collect();
    if parts.len() != 2 {
        return None;
    }
    let prefix: u8 = parts[1].parse().ok()?;
    if prefix == 0 {
        return None;
    }
    Some(format!("{}/{}", parts[0], prefix - 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widen_subnet_24_to_23() {
        assert_eq!(
            widen_subnet("192.168.100.0/24"),
            Some("192.168.100.0/23".into())
        );
    }

    #[test]
    fn widen_subnet_16_to_15() {
        assert_eq!(widen_subnet("172.27.0.0/16"), Some("172.27.0.0/15".into()));
    }

    #[test]
    fn managed_route_hint_returns_wider() {
        assert_eq!(managed_route_hint("192.168.1.0/24"), "192.168.1.0/23");
    }
}
