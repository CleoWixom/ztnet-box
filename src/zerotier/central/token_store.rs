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
    /// Cached client for the active token. Rebuilt only when the active token
    /// changes (set_active / add / remove / update). This prevents spawning a
    /// new RateLimiter refill task (tokio::spawn) on every incoming request.
    cached_client: Option<(String, ZtCentralClient)>, // (token_id, client)
}

impl TokenStore {
    pub fn new(tokens: Vec<CentralToken>, active_token_id: String) -> Self {
        Self {
            base_url: "https://api.zerotier.com/api/v1".into(),
            inner: Arc::new(RwLock::new(TokenStoreInner {
                tokens,
                active_token_id,
                cached_client: None,
            })),
        }
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }

    /// Returns the client for the active token, reusing the cached instance
    /// when the active token has not changed. This ensures the RateLimiter's
    /// background refill task is created at most once per token, not once per
    /// request.
    pub async fn active_client(&self) -> Option<ZtCentralClient> {
        // Fast path: check cache under read lock
        {
            let inner = self.inner.read().await;
            if let Some((ref cached_id, ref client)) = inner.cached_client {
                if *cached_id == inner.active_token_id {
                    return Some(client.clone());
                }
            }
        }

        // Slow path: rebuild under write lock
        let mut inner = self.inner.write().await;
        // Re-check after acquiring write lock (another task may have rebuilt it)
        if let Some((ref cached_id, ref client)) = inner.cached_client {
            if *cached_id == inner.active_token_id {
                return Some(client.clone());
            }
        }
        let token = inner
            .tokens
            .iter()
            .find(|t| t.id == inner.active_token_id)?;
        let client = ZtCentralClient::new(
            self.base_url.clone(),
            token.token.clone(),
            &token.rate_limit,
        );
        inner.cached_client = Some((token.id.clone(), client.clone()));
        Some(client)
    }

    /// Invalidate the cached client so it is rebuilt on the next call to
    /// `active_client`. Call after any mutation that changes the active token.
    async fn invalidate_cache(inner: &mut TokenStoreInner) {
        inner.cached_client = None;
    }

    pub async fn add(&self, name: String, token: String, rate_limit: RateLimit) -> CentralToken {
        let t = CentralToken::new(name, token, rate_limit);
        let mut inner = self.inner.write().await;
        if inner.active_token_id.is_empty() {
            inner.active_token_id = t.id.clone();
        }
        inner.tokens.push(t.clone());
        Self::invalidate_cache(&mut inner).await;
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
        let removed = inner.tokens.len() < before;
        if removed {
            Self::invalidate_cache(&mut inner).await;
        }
        removed
    }

    pub async fn set_active(&self, id: &str) -> bool {
        let mut inner = self.inner.write().await;
        if inner.tokens.iter().any(|t| t.id == id) {
            inner.active_token_id = id.to_string();
            Self::invalidate_cache(&mut inner).await;
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

    /// Update name/token/rate_limit in-place, preserving the original UUID.
    /// Returns the updated token, or `None` if `id` was not found.
    pub async fn update(
        &self,
        id: &str,
        name: String,
        token: String,
        rate_limit: RateLimit,
    ) -> Option<CentralToken> {
        let mut inner = self.inner.write().await;
        let t = inner.tokens.iter_mut().find(|t| t.id == id)?;
        t.name = name;
        t.token = token;
        t.rate_limit = rate_limit;
        let updated = t.clone();
        // Token value or rate_limit may have changed — rebuild client
        Self::invalidate_cache(&mut inner).await;
        Some(updated)
    }
}
