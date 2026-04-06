use super::types::{ZtLocalStatus, ZtNetwork};
use reqwest::Client;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LocalClientError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Auth token read error: {0}")]
    Token(String),
}

pub type Result<T> = std::result::Result<T, LocalClientError>;

#[derive(Clone)]
pub struct ZtLocalClient {
    client: Client,
    api_url: String,
    token: String,
}

impl ZtLocalClient {
    pub fn new(api_url: String, token_file: &std::path::Path) -> Result<Self> {
        let token = std::fs::read_to_string(token_file)
            .map(|s| s.trim().to_string())
            .map_err(|e| LocalClientError::Token(e.to_string()))?;
        let client = Client::builder()
            .danger_accept_invalid_certs(true) // ZT local uses self-signed
            .build()
            .map_err(LocalClientError::Http)?;
        Ok(Self {
            client,
            api_url,
            token,
        })
    }

    pub async fn status(&self) -> Result<ZtLocalStatus> {
        let url = format!("{}/status", self.api_url);
        let resp = self
            .client
            .get(&url)
            .header("X-ZT1-Auth", &self.token)
            .send()
            .await?
            .json()
            .await?;
        Ok(resp)
    }

    pub async fn networks(&self) -> Result<Vec<ZtNetwork>> {
        let url = format!("{}/network", self.api_url);
        let resp = self
            .client
            .get(&url)
            .header("X-ZT1-Auth", &self.token)
            .send()
            .await?
            .json()
            .await?;
        Ok(resp)
    }
}
