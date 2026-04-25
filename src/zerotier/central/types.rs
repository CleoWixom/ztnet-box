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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CentralMember {
    #[serde(rename = "nodeId")]
    pub node_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub authorized: bool,
    #[serde(rename = "activeBridge", default)]
    pub active_bridge: bool,
    #[serde(rename = "noAutoAssignIps", default)]
    pub no_auto_assign_ips: bool,
    #[serde(rename = "ipAssignments", default)]
    pub ip_assignments: Vec<String>,
    #[serde(default)]
    pub capabilities: Vec<i64>,
    #[serde(default)]
    pub tags: Vec<Vec<i64>>,
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
    #[serde(rename = "ssoExempt", default)]
    pub sso_exempt: bool,
    pub identity: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CentralMemberUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
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
}

// ── Account ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentralUser {
    pub id: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub email: String,
    #[serde(rename = "smsNumber")]
    pub sms_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountStatus {
    #[serde(default)]
    pub id: String,
    #[serde(rename = "displayName", default)]
    pub display_name: String,
    pub email: Option<String>,
    pub auth: Option<serde_json::Value>,
    #[serde(rename = "underLimit", default)]
    pub under_limit: bool,
    #[serde(rename = "planType")]
    pub plan_type: Option<String>,
}

impl AccountStatus {
    /// Определяет RateLimit по plan_type из Central API
    pub fn rate_limit(&self) -> RateLimit {
        match self.plan_type.as_deref() {
            Some("paid") | Some("business") | Some("enterprise") => RateLimit::Paid,
            _ => RateLimit::Free,
        }
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
