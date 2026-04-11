//! Layer 2 Bridge — bridges ZeroTier and a physical interface via Linux
//! `ip link` + systemd-networkd `.netdev` / `.network` files.
//!
//! Docs: https://docs.zerotier.com/bridging/
//!
//! Overview:
//!   1. Create `br0` bridge device (via `ip link`)
//!   2. Add zt_iface and phy_iface as bridge members
//!   3. Write systemd-networkd unit files for persistent config
//!   4. Optionally assign a static IP / gateway on br0
//!   5. Set `BridgeEnabled=true` in ZeroTier network config (via UI instruction)

pub mod deps;
pub mod platform;
pub mod rules;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ── Config & State ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConfig {
    /// ZeroTier interface (e.g. "zt7nnig26")
    pub zt_iface: String,
    /// Physical interface to bridge with ZT (e.g. "eth0")
    pub phy_iface: String,
    /// Bridge interface name (default: "br0")
    pub bridge_iface: String,
    /// Optional static IP for the bridge (CIDR, e.g. "192.168.1.10/24")
    pub bridge_addr: Option<String>,
    /// Optional default gateway for the bridge
    pub gateway: Option<String>,
    /// ZeroTier network ID (needed for instructions)
    pub network_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BridgeState {
    pub enabled: bool,
    pub config: Option<BridgeConfig>,
    pub applied_at: Option<DateTime<Utc>>,
}
