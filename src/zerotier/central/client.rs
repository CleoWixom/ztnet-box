use super::types::*;
use crate::{config::schema::RateLimit, server::error::ApiError};
use reqwest::{Client, Method, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use std::{sync::Arc, time::Duration};
use tokio::{sync::Semaphore, task::JoinHandle};

// ── Rate limiter ──────────────────────────────────────────────────────────────

/// Простой rate limiter через семафор + периодический release.
/// Free: 20 req/s, Paid: 100 req/s.
///
/// The refill task is stored as a `JoinHandle` and aborted on `Drop`
/// so clients created during e.g. `probe_token()` don't leak background tasks.
#[derive(Clone)]
struct RateLimiter {
    semaphore: Arc<Semaphore>,
    /// Refill task handle — aborted on Drop to prevent resource leaks.
    _refill_task: Arc<JoinHandle<()>>,
}

impl RateLimiter {
    fn new(rate_limit: &RateLimit) -> Self {
        let max = match rate_limit {
            RateLimit::Free => 20,
            RateLimit::Paid => 100,
        };
        let semaphore = Arc::new(Semaphore::new(max));
        let sem2 = Arc::clone(&semaphore);
        // Каждую секунду возвращаем разрешения обратно
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                let available = sem2.available_permits();
                let to_add = max.saturating_sub(available);
                if to_add > 0 {
                    sem2.add_permits(to_add);
                }
            }
        });
        Self {
            semaphore,
            _refill_task: Arc::new(handle),
        }
    }

    async fn acquire(&self) {
        // Consume the permit via forget() so the token is permanently removed
        // from the semaphore. The refill task tops up permits once per second,
        // giving true "max req/s" semantics. Without forget() the permit is
        // returned immediately on drop and the semaphore never drains —
        // allowing unlimited throughput.
        Arc::clone(&self.semaphore)
            .acquire_owned()
            .await
            .expect("rate-limiter semaphore closed")
            .forget();
    }
}

// ── Client ────────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct ZtCentralClient {
    pub base_url: String,
    token: String,
    http: Client,
    rate_limiter: RateLimiter,
}

impl ZtCentralClient {
    pub fn new(base_url: String, token: String, rate_limit: &RateLimit) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            token,
            http: Client::new(),
            rate_limiter: RateLimiter::new(rate_limit),
        }
    }

    // ── Internal request helper ───────────────────────────────────────────────

    async fn request<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<&impl Serialize>,
    ) -> Result<T, ApiError> {
        self.rate_limiter.acquire().await;

        let url = format!("{}{}", self.base_url, path);
        let mut req = self.http.request(method, &url).bearer_auth(&self.token);

        if let Some(b) = body {
            req = req.json(b);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| ApiError::ZtCentral(e.to_string()))?;

        let status = resp.status();

        if status == StatusCode::UNAUTHORIZED {
            return Err(ApiError::ZtCentral(
                "AUTH_FAILED: invalid or expired token".into(),
            ));
        }
        if status == StatusCode::NOT_FOUND {
            return Err(ApiError::NotFound);
        }
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(ApiError::ZtCentral(format!("Central API {status}: {text}")));
        }

        resp.json::<T>()
            .await
            .map_err(|e| ApiError::ZtCentral(format!("Deserialize: {e}")))
    }

    async fn request_empty(&self, method: Method, path: &str) -> Result<(), ApiError> {
        self.rate_limiter.acquire().await;

        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .request(method, &url)
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|e| ApiError::ZtCentral(e.to_string()))?;

        let status = resp.status();
        if status == StatusCode::UNAUTHORIZED {
            return Err(ApiError::ZtCentral("AUTH_FAILED".into()));
        }
        if status.is_success() || status == StatusCode::NO_CONTENT {
            Ok(())
        } else {
            let text = resp.text().await.unwrap_or_default();
            Err(ApiError::ZtCentral(format!("Central API {status}: {text}")))
        }
    }

    // ── Networks ──────────────────────────────────────────────────────────────

    pub async fn networks(&self) -> Result<Vec<CentralNetwork>, ApiError> {
        self.request(Method::GET, "/network", None::<&()>).await
    }

    pub async fn create_network(
        &self,
        cfg: &NetworkCreateOrUpdate,
    ) -> Result<CentralNetwork, ApiError> {
        self.request(Method::POST, "/network", Some(cfg)).await
    }

    pub async fn network(&self, id: &str) -> Result<CentralNetwork, ApiError> {
        self.request(Method::GET, &format!("/network/{id}"), None::<&()>)
            .await
    }

    pub async fn update_network(
        &self,
        id: &str,
        cfg: &NetworkCreateOrUpdate,
    ) -> Result<CentralNetwork, ApiError> {
        self.request(Method::POST, &format!("/network/{id}"), Some(cfg))
            .await
    }

    pub async fn delete_network(&self, id: &str) -> Result<(), ApiError> {
        self.request_empty(Method::DELETE, &format!("/network/{id}"))
            .await
    }

    // ── Members ───────────────────────────────────────────────────────────────

    pub async fn network_members(&self, net_id: &str) -> Result<Vec<CentralMember>, ApiError> {
        self.request(
            Method::GET,
            &format!("/network/{net_id}/member"),
            None::<&()>,
        )
        .await
    }

    pub async fn network_member(
        &self,
        net_id: &str,
        node_id: &str,
    ) -> Result<CentralMember, ApiError> {
        self.request(
            Method::GET,
            &format!("/network/{net_id}/member/{node_id}"),
            None::<&()>,
        )
        .await
    }

    pub async fn update_member(
        &self,
        net_id: &str,
        node_id: &str,
        update: &CentralMemberUpdate,
    ) -> Result<CentralMember, ApiError> {
        self.request(
            Method::PUT,
            &format!("/network/{net_id}/member/{node_id}"),
            Some(update),
        )
        .await
    }

    pub async fn delete_member(&self, net_id: &str, node_id: &str) -> Result<(), ApiError> {
        self.request_empty(
            Method::DELETE,
            &format!("/network/{net_id}/member/{node_id}"),
        )
        .await
    }

    // ── Account ───────────────────────────────────────────────────────────────

    pub async fn user(&self) -> Result<CentralUser, ApiError> {
        self.request(Method::GET, "/auth", None::<&()>).await
    }

    pub async fn account_status(&self) -> Result<AccountStatus, ApiError> {
        self.request(Method::GET, "/status", None::<&()>).await
    }

    pub async fn create_api_token(&self, name: &str) -> Result<ApiTokenRecord, ApiError> {
        #[derive(Serialize)]
        struct Body<'a> {
            token_name: &'a str,
        }
        self.request(
            Method::POST,
            "/auth/token",
            Some(&Body { token_name: name }),
        )
        .await
    }

    pub async fn delete_api_token(&self, token_id: &str) -> Result<(), ApiError> {
        self.request_empty(Method::DELETE, &format!("/auth/token/{token_id}"))
            .await
    }

    pub async fn random_token(&self) -> Result<String, ApiError> {
        #[derive(serde::Deserialize)]
        struct Resp {
            token: String,
        }
        let r: Resp = self
            .request(Method::GET, "/randomToken", None::<&()>)
            .await?;
        Ok(r.token)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn free_capacity() -> usize {
        match RateLimit::Free {
            RateLimit::Free => 20,
            RateLimit::Paid => 100,
        }
    }

    fn paid_capacity() -> usize {
        match RateLimit::Paid {
            RateLimit::Free => 20,
            RateLimit::Paid => 100,
        }
    }

    #[test]
    fn rate_limit_free_capacity() {
        assert_eq!(free_capacity(), 20);
    }

    #[test]
    fn rate_limit_paid_capacity() {
        assert_eq!(paid_capacity(), 100);
    }

    #[tokio::test]
    async fn rate_limiter_acquires_within_runtime() {
        let rl = RateLimiter::new(&RateLimit::Free);
        // Should be able to acquire immediately (permits available)
        tokio::time::timeout(std::time::Duration::from_millis(100), rl.acquire())
            .await
            .expect("acquire should not block when permits available");
    }

    #[tokio::test]
    async fn rate_limiter_blocks_when_exhausted() {
        // Create a limiter with only 2 permits so exhaustion is fast
        let semaphore = Arc::new(Semaphore::new(2));
        let rl = RateLimiter {
            semaphore,
            _refill_task: Arc::new(tokio::spawn(async {})),
        };
        // Drain all permits
        rl.acquire().await;
        rl.acquire().await;
        // Third acquire must block — timeout proves it
        let blocked =
            tokio::time::timeout(std::time::Duration::from_millis(50), rl.acquire()).await;
        assert!(
            blocked.is_err(),
            "acquire must block when no permits remain"
        );
    }

    #[test]
    fn account_status_rate_limit_paid() {
        let s = AccountStatus {
            id: "x".into(),
            display_name: "x".into(),
            email: None,
            auth: None,
            under_limit: true,
            plan_type: Some("paid".into()),
        };
        assert!(matches!(s.rate_limit(), RateLimit::Paid));
    }

    #[test]
    fn account_status_rate_limit_free() {
        let s = AccountStatus {
            id: "x".into(),
            display_name: "x".into(),
            email: None,
            auth: None,
            under_limit: true,
            plan_type: Some("free".into()),
        };
        assert!(matches!(s.rate_limit(), RateLimit::Free));
    }

    #[test]
    fn account_status_rate_limit_none() {
        let s = AccountStatus {
            id: "x".into(),
            display_name: "x".into(),
            email: None,
            auth: None,
            under_limit: true,
            plan_type: None,
        };
        assert!(matches!(s.rate_limit(), RateLimit::Free));
    }
}
