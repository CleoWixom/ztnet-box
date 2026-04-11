use crate::{
    bridge::{deps, platform, rules, BridgeConfig, BridgeState},
    server::{error::ApiError, state::AppState, validate},
};
use axum::{extract::State, response::IntoResponse, Json};
use serde::Deserialize;

// ── GET /api/bridge/platform ──────────────────────────────────────────────────

pub async fn get_platform(_: State<AppState>) -> impl IntoResponse {
    Json(platform::check())
}

// ── GET /api/bridge/deps ──────────────────────────────────────────────────────

pub async fn get_deps(_: State<AppState>) -> impl IntoResponse {
    Json(deps::check())
}

// ── POST /api/bridge/deps/install ─────────────────────────────────────────────

pub async fn install_deps(State(_s): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    #[cfg(unix)]
    if !nix::unistd::getuid().is_root() {
        return Err(ApiError::ZtLocal("Root required to install deps".into()));
    }
    deps::install(true)
        .map(Json)
        .map_err(ApiError::ZtLocal)
}

// ── GET /api/bridge/status ────────────────────────────────────────────────────

pub async fn get_status(State(s): State<AppState>) -> impl IntoResponse {
    Json(s.bridge_state.read().await.clone())
}

// ── POST /api/bridge/enable ───────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct EnableRequest {
    pub zt_iface: String,
    pub phy_iface: String,
    #[serde(default = "default_bridge_iface")]
    pub bridge_iface: String,
    pub bridge_addr: Option<String>,
    pub gateway: Option<String>,
    pub network_id: String,
}

fn default_bridge_iface() -> String {
    "br0".into()
}

pub async fn enable(
    State(s): State<AppState>,
    Json(req): Json<EnableRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Validation
    if req.zt_iface.trim().is_empty() {
        return Err(ApiError::InvalidInput("zt_iface required".into()));
    }
    if req.phy_iface.trim().is_empty() {
        return Err(ApiError::InvalidInput("phy_iface required".into()));
    }
    if req.bridge_iface.trim().is_empty() {
        return Err(ApiError::InvalidInput("bridge_iface required".into()));
    }
    validate::network_id(&req.network_id)?;
    if let Some(ref addr) = req.bridge_addr {
        validate::cidr(addr)?;
    }
    if let Some(ref gw) = req.gateway {
        validate::ip_addr(gw)?;
    }

    // Root check
    #[cfg(unix)]
    if !nix::unistd::getuid().is_root() {
        return Err(ApiError::ZtLocal(
            "Root privileges required for bridge setup".into(),
        ));
    }

    // Conflict: physnet active?
    let physnet_on = s.physnet_state.read().await.enabled;
    if physnet_on {
        return Err(ApiError::ZtLocal(
            "Physical Network Routing is active — disable it before enabling Bridge".into(),
        ));
    }

    let cfg = BridgeConfig {
        zt_iface: req.zt_iface,
        phy_iface: req.phy_iface,
        bridge_iface: req.bridge_iface,
        bridge_addr: req.bridge_addr,
        gateway: req.gateway,
        network_id: req.network_id.clone(),
    };

    rules::apply(&cfg).map_err(|e| ApiError::ZtLocal(e.to_string()))?;

    let state = BridgeState {
        enabled: true,
        config: Some(cfg),
        applied_at: Some(chrono::Utc::now()),
    };
    *s.bridge_state.write().await = state.clone();

    Ok(Json(serde_json::json!({
        "status": "enabled",
        "state": state,
        "next_step": format!(
            "In ZeroTier Central network {}, set 'bridging=true' for this member \
             so it can forward L2 frames between ZT and physical hosts.",
            req.network_id
        ),
    })))
}

// ── POST /api/bridge/disable ──────────────────────────────────────────────────

pub async fn disable(State(s): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    #[cfg(unix)]
    if !nix::unistd::getuid().is_root() {
        return Err(ApiError::ZtLocal("Root privileges required".into()));
    }

    let st = s.bridge_state.read().await.clone();
    if let Some(cfg) = st.config {
        rules::remove(&cfg).map_err(|e| ApiError::ZtLocal(e.to_string()))?;
    }
    *s.bridge_state.write().await = BridgeState::default();
    Ok(Json(serde_json::json!({ "status": "disabled" })))
}
