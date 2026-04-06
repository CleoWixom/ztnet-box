use super::types::{CentralMember, CentralNetwork};
use reqwest::Client;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CentralClientError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("API error {status}: {message}")]
    Api { status: u16, message: String },
}

pub type Result<T> = std::result::Result<T, CentralClientError>;

#[derive(Clone)]
pub struct ZtCentralClient {
    client: Client,
    base_url: String,
    token: String,
}

impl ZtCentralClient {
    pub fn new(base_url: String, token: String) -> Self {
        let client = Client::new();
        Self {
            client,
            base_url,
            token,
        }
    }

    pub async fn networks(&self) -> Result<Vec<CentralNetwork>> {
        let url = format!("{}/network", self.base_url);
        Ok(self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn members(&self, network_id: &str) -> Result<Vec<CentralMember>> {
        let url = format!("{}/network/{}/member", self.base_url, network_id);
        Ok(self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await?
            .json()
            .await?)
    }
}
