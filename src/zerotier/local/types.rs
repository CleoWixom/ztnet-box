use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZtLocalStatus {
    pub address: String,
    pub version: String,
    pub online: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZtNetwork {
    pub id: String,
    pub name: String,
    pub status: String,
    pub r#type: String,
    #[serde(rename = "assignedAddresses")]
    pub assigned_addresses: Vec<String>,
}
