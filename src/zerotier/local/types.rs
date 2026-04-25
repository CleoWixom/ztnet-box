use serde::{Deserialize, Serialize};

// ── Node Status ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeStatus {
    pub address: String,
    pub public_identity: String,
    pub world_id: Option<u64>,
    pub cluster_node_id: Option<u32>,
    pub clock: u64,
    pub online: bool,
    pub tcp_fallback_active: bool,
    pub relay_policy: Option<String>,
    pub version: String,
}

// ── Network Membership ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkMembership {
    pub id: String,
    pub name: String,
    pub status: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub mac: String,
    pub mtu: u32,
    pub dhcp: bool,
    pub bridge: bool,
    pub broadcast_enabled: bool,
    pub port_error: i32,
    pub netconf_revision: u32,
    pub assigned_addresses: Vec<String>,
    pub routes: Vec<Route>,
    pub dns: Option<Dns>,
    pub allow_managed: bool,
    pub allow_global: bool,
    pub allow_default: bool,
    pub allow_dns: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NetworkMembershipUpdate {
    pub allow_managed: Option<bool>,
    pub allow_global: Option<bool>,
    pub allow_default: Option<bool>,
    pub allow_dns: Option<bool>,
    pub dns: Option<Dns>,
}

// ── Peers ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeerInfo {
    pub address: String,
    pub version: Option<String>,
    pub role: String,
    pub latency: i32,
    pub last_unicast_frame: Option<u64>,
    pub last_multicast_frame: Option<u64>,
    pub paths: Vec<PeerPath>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeerPath {
    pub address: String,
    pub last_send: u64,
    pub last_receive: u64,
    pub active: bool,
    pub expired: bool,
    pub preferred: bool,
    pub trusted_path_id: Option<u64>,
}

// ── Controller ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControllerNetwork {
    pub id: String,
    pub name: String,
    pub private: bool,
    pub creation_time: u64,
    pub routes: Vec<Route>,
    #[serde(rename = "ipAssignmentPools")]
    pub ip_assignment_pools: Vec<IpRange>,
    pub v4_assign_mode: Option<V4AssignMode>,
    pub v6_assign_mode: Option<V6AssignMode>,
    pub mtu: u32,
    pub multicast_limit: u32,
    pub enable_broadcast: bool,
    pub dns: Option<Dns>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControllerNetworkCreate {
    pub name: Option<String>,
    pub private: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControllerMember {
    pub node_id: String,
    pub authorized: bool,
    pub active_bridge: bool,
    pub ip_assignments: Vec<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub no_auto_assign_ips: bool,
    pub sso_exempt: bool,
    pub revision: u64,
    pub last_modified_time: u64,
    pub capabilities: Vec<i64>,
    pub tags: Vec<Vec<i64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ControllerMemberUpdate {
    pub authorized: Option<bool>,
    pub active_bridge: Option<bool>,
    pub ip_assignments: Option<Vec<String>>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub no_auto_assign_ips: Option<bool>,
}

// ── Moons ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Moon {
    pub id: String,
    pub roots: Vec<MoonRoot>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoonRoot {
    pub identity: String,
    #[serde(rename = "stableEndpoints")]
    pub stable_endpoints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrbitRequest {
    pub seed: Option<String>,
}

// ── Shared ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub target: String,
    pub via: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)] // ZT returns {} when DNS is unconfigured — default all fields
pub struct Dns {
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub servers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpRange {
    #[serde(rename = "ipRangeStart")]
    pub start: String,
    #[serde(rename = "ipRangeEnd")]
    pub end: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V4AssignMode {
    pub zt: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V6AssignMode {
    pub zt: bool,
    pub rfc4193: bool,
    pub plan6: bool,
}
