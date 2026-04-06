use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentralNetwork {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "totalMemberCount")]
    pub total_member_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentralMember {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub authorized: bool,
    pub online: Option<bool>,
}
