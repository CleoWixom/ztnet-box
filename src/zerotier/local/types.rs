use serde::{Deserialize, Deserializer, Serialize};

// ── Node Status ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct NodeStatus {
    pub address: String,
    pub public_identity: String,
    /// Renamed from world_id — spec: planetWorldId (ZT-C-1)
    #[serde(rename = "planetWorldId")]
    pub planet_world_id: Option<u64>,
    #[serde(rename = "planetWorldTimestamp")]
    pub planet_world_timestamp: Option<u64>,
    pub cluster_node_id: Option<u32>,
    pub clock: u64,
    pub online: bool,
    pub tcp_fallback_active: bool,
    pub relay_policy: Option<String>,
    pub version: String,
    /// Version breakdown (ZT-C-1)
    pub version_major: Option<i32>,
    pub version_minor: Option<i32>,
    pub version_rev: Option<i32>,
    pub version_build: Option<i32>,
    pub config: Option<NodeConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct NodeConfig {
    pub settings: Option<NodeSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct NodeSettings {
    pub allow_tcp_fallback_relay: bool,
    pub primary_port: Option<u16>,
    pub secondary_port: Option<u16>,
    pub tertiary_port: Option<u16>,
    pub surface_addresses: Vec<String>,
}

// ── Network Membership ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
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
    /// ZT returns {} or [] for empty DNS — custom deserializer handles both (ZT-C-2)
    #[serde(default, deserialize_with = "deserialize_dns_flex")]
    pub dns: Option<Dns>,
    pub allow_managed: bool,
    pub allow_global: bool,
    pub allow_default: bool,
    pub allow_dns: bool,
    /// Name of the OS TUN/TAP interface, e.g. "zt3jnwgx6c" (ZT-C-3)
    pub port_device_name: Option<String>,
    /// SSO authentication URL if network requires SSO (ZT-C-3)
    pub authentication_url: Option<String>,
    pub authentication_expiry_time: Option<u64>,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct PeerInfo {
    pub address: String,
    pub version: Option<String>,
    pub role: String,
    /// spec: uSafeint | -1; use i64 to avoid overflow (ZT-C-4)
    pub latency: i64,
    pub last_unicast_frame: Option<u64>,
    pub last_multicast_frame: Option<u64>,
    pub paths: Vec<PeerPath>,
    /// Whether this peer is reached via a relay/tunnel (ZT-C-4)
    pub tunneled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct PeerPath {
    pub address: String,
    pub last_send: u64,
    pub last_receive: u64,
    pub active: bool,
    pub expired: bool,
    pub preferred: bool,
    pub trusted_path_id: Option<u64>,
    /// Local socket descriptor (ZT-C-4)
    pub local_socket: Option<u64>,
}

// ── Controller ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ControllerNetwork {
    pub id: String,
    /// Duplicate of id — ZT always returns both (ZT-C-5)
    #[serde(rename = "nwid")]
    pub nwid: String,
    pub name: String,
    pub private: bool,
    pub creation_time: u64,
    pub revision: u64,
    pub routes: Vec<Route>,
    #[serde(rename = "ipAssignmentPools")]
    pub ip_assignment_pools: Vec<IpRange>,
    pub v4_assign_mode: Option<V4AssignMode>,
    pub v6_assign_mode: Option<V6AssignMode>,
    pub mtu: u32,
    pub multicast_limit: u32,
    pub enable_broadcast: bool,
    pub dns: Option<Dns>,
    pub capabilities: Vec<serde_json::Value>,
    pub rules: Vec<serde_json::Value>,
    pub tags: Vec<serde_json::Value>,
}

/// Full request body for creating or updating a controller network (ZT-C-8)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ControllerNetworkRequest {
    pub name: Option<String>,
    pub private: Option<bool>,
    pub enable_broadcast: Option<bool>,
    pub mtu: Option<u32>,
    pub multicast_limit: Option<u32>,
    pub ip_assignment_pools: Option<Vec<IpRange>>,
    pub routes: Option<Vec<Route>>,
    pub v4_assign_mode: Option<V4AssignMode>,
    pub v6_assign_mode: Option<V6AssignMode>,
    pub dns: Option<Dns>,
}

/// Legacy alias kept for handler compatibility — maps to ControllerNetworkRequest
pub type ControllerNetworkCreate = ControllerNetworkRequest;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ControllerMember {
    /// Local controller uses "id" for member address, not "nodeId" (ZT-C-6)
    /// Accept both field names for compatibility
    #[serde(alias = "nodeId", alias = "address")]
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
#[serde(default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct V4AssignMode {
    pub zt: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct V6AssignMode {
    pub zt: bool,
    pub rfc4193: bool,
    /// spec field name is "6plane" — starts with digit, must use rename (ZT-C-7)
    #[serde(rename = "6plane", default)]
    pub six_plane: bool,
}

// ── Custom deserializers ──────────────────────────────────────────────────────

/// ZeroTier returns `dns` as either:
///   - `{"domain":"...","servers":[...]}` (configured)
///   - `{}`                              (empty object, unconfigured)
///   - `[]`                              (empty array, some versions)
///
/// This deserializer handles all three cases. (ZT-C-2)
fn deserialize_dns_flex<'de, D>(d: D) -> Result<Option<Dns>, D::Error>
where
    D: Deserializer<'de>,
{
    let v: serde_json::Value = serde_json::Value::deserialize(d)?;
    match v {
        serde_json::Value::Null => Ok(None),
        serde_json::Value::Array(_) => Ok(None), // empty [] → no DNS
        serde_json::Value::Object(ref map) if map.is_empty() => Ok(None), // {} → no DNS
        other => {
            let dns: Dns = serde_json::from_value(other).unwrap_or_default();
            if dns.domain.is_empty() && dns.servers.is_empty() {
                Ok(None)
            } else {
                Ok(Some(dns))
            }
        }
    }
}
