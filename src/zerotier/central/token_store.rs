use crate::config::schema::{CentralToken, RateLimit};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::client::ZtCentralClient;

#[derive(Clone)]
pub struct TokenStore {
    inner: Arc<RwLock<TokenStoreInner>>,
    base_url: String,
}

struct TokenStoreInner {
    tokens: Vec<CentralToken>,
    active_token_id: String,
}

impl TokenStore {
    pub fn new(tokens: Vec<CentralToken>, active_token_id: String) -> Self {
        Self {
            base_url: "https://api.zerotier.com/api/v1".into(),
            inner: Arc::new(RwLock::new(TokenStoreInner {
                tokens,
                active_token_id,
            })),
        }
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }

    /// Возвращает клиент активного токена или None если нет активного.
    pub async fn active_client(&self) -> Option<ZtCentralClient> {
        let inner = self.inner.read().await;
        let token = inner
            .tokens
            .iter()
            .find(|t| t.id == inner.active_token_id)?;
        Some(ZtCentralClient::new(
            self.base_url.clone(),
            token.token.clone(),
            &token.rate_limit,
        ))
    }

    pub async fn add(&self, name: String, token: String, rate_limit: RateLimit) -> CentralToken {
        let t = CentralToken::new(name, token, rate_limit);
        let mut inner = self.inner.write().await;
        if inner.active_token_id.is_empty() {
            inner.active_token_id = t.id.clone();
        }
        inner.tokens.push(t.clone());
        t
    }

    pub async fn remove(&self, id: &str) -> bool {
        let mut inner = self.inner.write().await;
        let before = inner.tokens.len();
        inner.tokens.retain(|t| t.id != id);
        if inner.active_token_id == id {
            inner.active_token_id = inner
                .tokens
                .first()
                .map(|t| t.id.clone())
                .unwrap_or_default();
        }
        inner.tokens.len() < before
    }

    pub async fn set_active(&self, id: &str) -> bool {
        let mut inner = self.inner.write().await;
        if inner.tokens.iter().any(|t| t.id == id) {
            inner.active_token_id = id.to_string();
            true
        } else {
            false
        }
    }

    pub async fn active_token(&self) -> Option<CentralToken> {
        let inner = self.inner.read().await;
        inner
            .tokens
            .iter()
            .find(|t| t.id == inner.active_token_id)
            .cloned()
    }

    pub async fn list(&self) -> Vec<CentralToken> {
        self.inner.read().await.tokens.clone()
    }

    pub async fn find(&self, id: &str) -> Option<CentralToken> {
        self.inner
            .read()
            .await
            .tokens
            .iter()
            .find(|t| t.id == id)
            .cloned()
    }
}
