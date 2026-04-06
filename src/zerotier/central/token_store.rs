use crate::config::schema::{CentralToken, RateLimit};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Потокобезопасное хранилище токенов Central API.
#[derive(Clone)]
pub struct TokenStore {
    inner: Arc<RwLock<TokenStoreInner>>,
}

struct TokenStoreInner {
    tokens: Vec<CentralToken>,
    active_token_id: String,
}

impl TokenStore {
    pub fn new(tokens: Vec<CentralToken>, active_token_id: String) -> Self {
        Self {
            inner: Arc::new(RwLock::new(TokenStoreInner {
                tokens,
                active_token_id,
            })),
        }
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

    pub async fn remove(&self, id: &str) {
        let mut inner = self.inner.write().await;
        inner.tokens.retain(|t| t.id != id);
        if inner.active_token_id == id {
            inner.active_token_id = inner
                .tokens
                .first()
                .map(|t| t.id.clone())
                .unwrap_or_default();
        }
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
        let active_id = &inner.active_token_id;
        inner.tokens.iter().find(|t| &t.id == active_id).cloned()
    }

    pub async fn list(&self) -> Vec<CentralToken> {
        self.inner.read().await.tokens.clone()
    }
}
