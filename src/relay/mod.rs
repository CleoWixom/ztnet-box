//! TCP Relay management.
//!
//! Two concerns:
//!  1. **Local config** — read/write `force_tcp_relay` and `tcp_fallback_relay`
//!     in ZeroTier's `local.conf` via the existing `local_config` module.
//!  2. **Remote deploy** — SSH into a remote host, install Docker, and start
//!     the `zerotier/pylon` reflect container.
//!
//! Docs: https://docs.zerotier.com/relay/

pub mod deploy;
pub mod ssh;

use serde::{Deserialize, Serialize};

// ── Types ─────────────────────────────────────────────────────────────────────

/// The full relay state returned by GET /api/relay/status.
#[derive(Debug, Clone, Serialize, Default)]
pub struct RelayStatus {
    /// Current local.conf relay settings (read from disk).
    pub local: LocalRelayConfig,
    /// Remote pylon container status (None if never deployed).
    pub remote: Option<RemoteRelayInfo>,
}

/// Local relay settings mirrored from local.conf.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocalRelayConfig {
    /// Force all ZeroTier traffic through TCP relay.
    pub force_tcp_relay: bool,
    /// Custom relay endpoint "ip/port", e.g. "1.2.3.4/443".
    pub tcp_fallback_relay: Option<String>,
}

/// State of a deployed remote pylon relay container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteRelayInfo {
    /// SSH host that was used for deployment.
    pub host: String,
    /// Port the pylon container listens on (default 443).
    pub port: u16,
    /// Whether the last reachability check succeeded.
    pub reachable: Option<bool>,
    /// RFC3339 timestamp of the last deploy.
    pub deployed_at: String,
}

/// Parameters for deploying a remote pylon relay.
#[derive(Debug, Clone, Deserialize)]
pub struct RelayDeployConfig {
    /// Remote host (IP or hostname).
    pub host: String,
    /// SSH port (default 22).
    #[serde(default = "default_ssh_port")]
    pub ssh_port: u16,
    /// SSH user (default "root").
    #[serde(default = "default_ssh_user")]
    pub ssh_user: String,
    /// Path to private key file on the *local* machine.
    /// If omitted, SSH uses the default key from `~/.ssh/`.
    pub key_path: Option<String>,
    /// Port for the pylon reflect container (default 443).
    #[serde(default = "default_pylon_port")]
    pub pylon_port: u16,
    /// Stop UFW before starting Docker (avoids iptables conflicts).
    #[serde(default = "default_true")]
    pub stop_ufw: bool,
}

fn default_ssh_port() -> u16 {
    22
}
fn default_ssh_user() -> String {
    "root".into()
}
fn default_pylon_port() -> u16 {
    443
}
fn default_true() -> bool {
    true
}
