use super::types::*;
use crate::{config::schema::LocalConfig, server::error::ApiError};
use reqwest::{Client, Method, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;

// ── Client ────────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct ZtLocalClient {
    pub base_url: String,
    token: String,
    http: Client,
}

impl ZtLocalClient {
    pub fn new(api_url: &str, token: &str) -> Self {
        // ZeroTier One uses a self-signed TLS cert on localhost — safe to skip
        // verification only when the URL resolves to a loopback address.
        // For remote api_url (rare but possible) we enforce cert validation.
        let is_loopback = api_url.contains("127.0.0.1")
            || api_url.contains("localhost")
            || api_url.contains("[::1]");
        let http = Client::builder()
            .danger_accept_invalid_certs(is_loopback)
            // Hard timeout: if ZeroTier daemon is not running or unreachable,
            // fail fast instead of hanging the browser with infinite spinners.
            .timeout(std::time::Duration::from_secs(5))
            .connect_timeout(std::time::Duration::from_secs(3))
            .build()
            .expect("reqwest client");
        Self {
            base_url: api_url.trim_end_matches('/').to_string(),
            token: token.to_string(),
            http,
        }
    }

    pub fn from_config(cfg: &LocalConfig) -> Result<Self, ApiError> {
        let token = std::fs::read_to_string(&cfg.token_file)
            .map(|s| s.trim().to_string())
            .map_err(|e| {
                ApiError::ZtLocal(format!(
                    "Cannot read auth token from {}: {e}",
                    cfg.token_file.display()
                ))
            })?;
        Ok(Self::new(&cfg.api_url, &token))
    }

    // ── Internal request helper ───────────────────────────────────────────────

    async fn request<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<&impl Serialize>,
    ) -> Result<T, ApiError> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self
            .http
            .request(method, &url)
            .header("X-ZT1-AUTH", &self.token);

        if let Some(b) = body {
            req = req.json(b);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| ApiError::ZtLocal(e.to_string()))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(ApiError::ZtLocal(format!("ZT Local {status}: {text}")));
        }

        resp.json::<T>()
            .await
            .map_err(|e| ApiError::ZtLocal(format!("Deserialize error: {e}")))
    }

    async fn request_empty(&self, method: Method, path: &str) -> Result<(), ApiError> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .request(method, &url)
            .header("X-ZT1-AUTH", &self.token)
            .send()
            .await
            .map_err(|e| ApiError::ZtLocal(e.to_string()))?;

        let status = resp.status();
        if status.is_success() || status == StatusCode::NO_CONTENT {
            Ok(())
        } else {
            let text = resp.text().await.unwrap_or_default();
            Err(ApiError::ZtLocal(format!("ZT Local {status}: {text}")))
        }
    }

    // ── Node ──────────────────────────────────────────────────────────────────

    pub async fn node_status(&self) -> Result<NodeStatus, ApiError> {
        self.request(Method::GET, "/status", None::<&()>).await
    }

    // ── Joined Networks ───────────────────────────────────────────────────────

    pub async fn networks(&self) -> Result<Vec<NetworkMembership>, ApiError> {
        self.request(Method::GET, "/network", None::<&()>).await
    }

    pub async fn network(&self, id: &str) -> Result<NetworkMembership, ApiError> {
        self.request(Method::GET, &format!("/network/{id}"), None::<&()>)
            .await
    }

    pub async fn join_network(
        &self,
        id: &str,
        update: &NetworkMembershipUpdate,
    ) -> Result<NetworkMembership, ApiError> {
        self.request(Method::POST, &format!("/network/{id}"), Some(update))
            .await
    }

    pub async fn leave_network(&self, id: &str) -> Result<(), ApiError> {
        // ZT-M-3: DELETE /network/{id} returns {"result":true} on success.
        // Parse response body to distinguish success from errors.
        let result: serde_json::Value = self
            .request(Method::DELETE, &format!("/network/{id}"), None::<&()>)
            .await?;
        if result["result"] == true || result["result"] == serde_json::Value::Null {
            Ok(())
        } else {
            Err(ApiError::ZtLocal(format!(
                "leave_network failed: {}",
                result
            )))
        }
    }

    // ── Peers ─────────────────────────────────────────────────────────────────

    pub async fn peers(&self) -> Result<Vec<PeerInfo>, ApiError> {
        self.request(Method::GET, "/peer", None::<&()>).await
    }

    pub async fn peer(&self, node_id: &str) -> Result<PeerInfo, ApiError> {
        self.request(Method::GET, &format!("/peer/{node_id}"), None::<&()>)
            .await
    }

    // ── Controller — Networks ─────────────────────────────────────────────────

    pub async fn controller_networks(&self) -> Result<Vec<String>, ApiError> {
        self.request(Method::GET, "/controller/network", None::<&()>)
            .await
    }

    pub async fn controller_network(&self, net_id: &str) -> Result<ControllerNetwork, ApiError> {
        self.request(
            Method::GET,
            &format!("/controller/network/{net_id}"),
            None::<&()>,
        )
        .await
    }

    pub async fn create_controller_network(
        &self,
        node_address: &str,
        cfg: &ControllerNetworkCreate,
    ) -> Result<ControllerNetwork, ApiError> {
        // ZT-M-2: POST /controller/network/{nodeid}______ — ZT generates
        // the random 6-char suffix server-side. Underscore padding fills to 16 chars.
        let net_id = format!("{node_address}______");
        self.request(
            Method::POST,
            &format!("/controller/network/{net_id}"),
            Some(cfg),
        )
        .await
    }

    pub async fn update_controller_network(
        &self,
        net_id: &str,
        cfg: &ControllerNetworkCreate,
    ) -> Result<ControllerNetwork, ApiError> {
        self.request(
            Method::POST,
            &format!("/controller/network/{net_id}"),
            Some(cfg),
        )
        .await
    }

    pub async fn delete_controller_network(&self, net_id: &str) -> Result<(), ApiError> {
        self.request_empty(Method::DELETE, &format!("/controller/network/{net_id}"))
            .await
    }

    // ── Controller — Members ──────────────────────────────────────────────────

    pub async fn network_members(&self, net_id: &str) -> Result<Vec<ControllerMember>, ApiError> {
        // Step 1: GET /controller/network/{id}/member → { node_id: revision, … }
        let ids: std::collections::HashMap<String, serde_json::Value> = self
            .request(
                Method::GET,
                &format!("/controller/network/{net_id}/member"),
                None::<&()>,
            )
            .await?;
        // Step 2: fetch all member details in parallel (ZT-C-9: was sequential N+1)
        let mut set = tokio::task::JoinSet::new();
        let client = std::sync::Arc::new(self.clone());
        for id in ids.into_keys() {
            let c   = client.clone();
            let nid = net_id.to_string();
            set.spawn(async move { c.network_member(&nid, &id).await });
        }
        let mut members = Vec::new();
        while let Some(res) = set.join_next().await {
            if let Ok(Ok(m)) = res { members.push(m); }
        }
        Ok(members)
    }

    /// ZT-L-4: GET /unstable/controller/network/{id}/member returns all member
    /// details in a single request — much faster than N individual fetches.
    /// Falls back to the stable N+1 approach if the endpoint returns 404 or errors.
    pub async fn network_members_bulk(
        &self,
        net_id: &str,
    ) -> Result<Vec<ControllerMember>, ApiError> {
        match self
            .request::<Vec<ControllerMember>>(
                Method::GET,
                &format!("/unstable/controller/network/{net_id}/member"),
                None::<&()>,
            )
            .await
        {
            Ok(members) => Ok(members),
            Err(_) => self.network_members(net_id).await, // fall back to stable N+1
        }
    }

    pub async fn network_member(
        &self,
        net_id: &str,
        node_id: &str,
    ) -> Result<ControllerMember, ApiError> {
        self.request(
            Method::GET,
            &format!("/controller/network/{net_id}/member/{node_id}"),
            None::<&()>,
        )
        .await
    }

    pub async fn update_member(
        &self,
        net_id: &str,
        node_id: &str,
        update: &ControllerMemberUpdate,
    ) -> Result<ControllerMember, ApiError> {
        self.request(
            Method::POST,
            &format!("/controller/network/{net_id}/member/{node_id}"),
            Some(update),
        )
        .await
    }

    pub async fn delete_member(&self, net_id: &str, node_id: &str) -> Result<(), ApiError> {
        self.request_empty(
            Method::DELETE,
            &format!("/controller/network/{net_id}/member/{node_id}"),
        )
        .await
    }

    // ── Moons ─────────────────────────────────────────────────────────────────

    pub async fn moons(&self) -> Result<Vec<Moon>, ApiError> {
        self.request(Method::GET, "/moon", None::<&()>).await
    }

    pub async fn orbit_moon(
        &self,
        world_id: &str,
        req: &OrbitRequest,
    ) -> Result<Vec<Moon>, ApiError> {
        self.request(Method::POST, &format!("/moon/{world_id}"), Some(req))
            .await
    }

    pub async fn deorbit_moon(&self, world_id: &str) -> Result<(), ApiError> {
        self.request_empty(Method::DELETE, &format!("/moon/{world_id}"))
            .await
    }
}
