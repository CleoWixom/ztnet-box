use crate::config::schema::RateLimit;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ── Networks ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentralNetwork {
    pub id: String,
    pub config: CentralNetworkConfig,
    pub description: Option<String>,
    #[serde(rename = "rulesSource")]
    pub rules_source: Option<String>,
    #[serde(rename = "ownerId")]
    pub owner_id: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<i64>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CentralNetworkConfig {
    pub name: Option<String>,
    pub private: Option<bool>,
    pub routes: Option<Vec<CentralRoute>>,
    #[serde(rename = "ipAssignmentPools")]
    pub ip_assignment_pools: Option<Vec<CentralIpRange>>,
    #[serde(rename = "v4AssignMode")]
    pub v4_assign_mode: Option<serde_json::Value>,
    #[serde(rename = "v6AssignMode")]
    pub v6_assign_mode: Option<serde_json::Value>,
    pub mtu: Option<u32>,
    #[serde(rename = "multicastLimit")]
    pub multicast_limit: Option<u32>,
    #[serde(rename = "enableBroadcast")]
    pub enable_broadcast: Option<bool>,
    pub dns: Option<CentralDns>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentralRoute {
    pub target: String,
    pub via: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentralIpRange {
    #[serde(rename = "ipRangeStart")]
    pub start: String,
    #[serde(rename = "ipRangeEnd")]
    pub end: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)] // ZT Central returns "dns":{} when unconfigured
pub struct CentralDns {
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub servers: Vec<String>,
}

/// Body for POST /network or POST /network/:id
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkCreateOrUpdate {
    pub config: Option<CentralNetworkConfig>,
}

// ── Members ───────────────────────────────────────────────────────────────────

/// Legacy Central API wraps mutable member fields inside a `config` object.
/// Top-level fields (name, networkId, lastOnline, physicalAddress, etc.) are
/// returned directly. Mutable fields (authorized, ipAssignments, …) live in
/// `config`. (ZT-C-11)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CentralMember {
    /// Member node address (10-hex-char ZeroTier address)
    #[serde(rename = "nodeId")]
    pub node_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "networkId")]
    pub network_id: Option<String>,
    #[serde(rename = "lastOnline")]
    pub last_online: Option<i64>,
    #[serde(rename = "physicalAddress")]
    pub physical_address: Option<String>,
    #[serde(rename = "clientVersion")]
    pub client_version: Option<String>,
    #[serde(rename = "protocolVersion")]
    pub protocol_version: Option<i32>,
    #[serde(rename = "supportsRulesEngine")]
    pub supports_rules_engine: Option<bool>,
    pub identity: Option<String>,
    /// The mutable fields are nested under "config" in Legacy Central API.
    /// Use #[serde(flatten)] on CentralMemberConfig when returning to frontend
    /// so the UI receives a flat object (ZT-C-11).
    pub config: CentralMemberConfig,
}

impl CentralMember {
    /// Convenience accessors that delegate to config (avoids m.config.authorized everywhere)
    pub fn authorized(&self) -> bool {
        self.config.authorized
    }
    pub fn ip_assignments(&self) -> &Vec<String> {
        &self.config.ip_assignments
    }
    pub fn active_bridge(&self) -> bool {
        self.config.active_bridge
    }
    pub fn no_auto_assign_ips(&self) -> bool {
        self.config.no_auto_assign_ips
    }
    pub fn sso_exempt(&self) -> bool {
        self.config.sso_exempt
    }
}

/// Flat view of CentralMember for serialization to the frontend.
/// Combines top-level and config fields into one flat JSON object.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CentralMemberView {
    pub node_id: String,
    pub name: Option<String>,
    pub network_id: Option<String>,
    pub last_online: Option<i64>,
    pub physical_address: Option<String>,
    pub client_version: Option<String>,
    // Flattened from config
    pub authorized: bool,
    pub active_bridge: bool,
    pub no_auto_assign_ips: bool,
    pub ip_assignments: Vec<String>,
    pub sso_exempt: bool,
    pub capabilities: Vec<i64>,
    pub tags: Vec<Vec<i64>>,
}

impl From<CentralMember> for CentralMemberView {
    fn from(m: CentralMember) -> Self {
        Self {
            node_id: m.node_id,
            name: m.name,
            network_id: m.network_id,
            last_online: m.last_online,
            physical_address: m.physical_address,
            client_version: m.client_version,
            authorized: m.config.authorized,
            active_bridge: m.config.active_bridge,
            no_auto_assign_ips: m.config.no_auto_assign_ips,
            ip_assignments: m.config.ip_assignments,
            sso_exempt: m.config.sso_exempt,
            capabilities: m.config.capabilities,
            tags: m.config.tags,
        }
    }
}

/// Fields nested under `member.config` in Legacy Central API responses.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct CentralMemberConfig {
    pub authorized: bool,
    pub active_bridge: bool,
    pub no_auto_assign_ips: bool,
    pub ip_assignments: Vec<String>,
    pub capabilities: Vec<i64>,
    pub tags: Vec<Vec<i64>>,
    pub sso_exempt: bool,
    /// ZT node address repeated inside config
    #[serde(rename = "id")]
    pub id: String,
    pub nwid: String,
}

/// Legacy Central API member update — wraps fields in {"config": {...}} (ZT-C-12)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CentralMemberUpdate {
    pub config: CentralMemberUpdateConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CentralMemberUpdateConfig {
    pub authorized: Option<bool>,
    #[serde(rename = "activeBridge")]
    pub active_bridge: Option<bool>,
    #[serde(rename = "noAutoAssignIps")]
    pub no_auto_assign_ips: Option<bool>,
    #[serde(rename = "ipAssignments")]
    pub ip_assignments: Option<Vec<String>>,
    pub capabilities: Option<Vec<i64>>,
    pub tags: Option<Vec<Vec<i64>>>,
    #[serde(rename = "ssoExempt")]
    pub sso_exempt: Option<bool>,
    pub name: Option<String>,
    pub description: Option<String>,
}

// ── Account ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CentralUser {
    pub id: String,
    #[serde(rename = "displayName", default)]
    pub display_name: String,
    #[serde(default)]
    pub email: String,
    #[serde(rename = "smsNumber")]
    pub sms_number: Option<String>,
    pub subscriptions: Option<serde_json::Value>,
}

/// Response from GET /self in Central API (ZT-C-14)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AccountStatus {
    #[serde(default)]
    pub id: String,
    #[serde(rename = "displayName", default)]
    pub display_name: String,
    pub email: Option<String>,
    pub auth: Option<serde_json::Value>,
    #[serde(rename = "underLimit", default)]
    pub under_limit: bool,
    /// Subscriptions object from /self — contains plan info (ZT-C-14)
    /// {"zerotier": {"planId": "free"|"paid"|"business", ...}}
    pub subscriptions: Option<serde_json::Value>,
}

impl AccountStatus {
    /// Determine rate limit from subscriptions.zerotier.planId (ZT-C-14)
    /// planType field does not exist in /self response — must read subscriptions
    pub fn rate_limit(&self) -> RateLimit {
        if let Some(subs) = &self.subscriptions {
            if let Some(plan_id) = subs
                .get("zerotier")
                .and_then(|z| z.get("planId"))
                .and_then(|p| p.as_str())
            {
                return match plan_id {
                    "paid" | "business" | "enterprise" | "pro" => RateLimit::Paid,
                    _ => RateLimit::Free,
                };
            }
            // Fallback: check top-level "plan" field some API versions use
            if let Some(plan) = subs.get("plan").and_then(|p| p.as_str()) {
                return match plan {
                    "paid" | "business" | "enterprise" | "pro" => RateLimit::Paid,
                    _ => RateLimit::Free,
                };
            }
        }
        RateLimit::Free
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiTokenRecord {
    pub id: String,
    #[serde(rename = "tokenName")]
    pub token_name: String,
    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "lastUsed")]
    pub last_used: Option<DateTime<Utc>>,
}
