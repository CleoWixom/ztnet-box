use crate::{
    exitnode::{deps, interfaces, platform, ExitNodeManager},
    server::{error::ApiError, state::AppState},
};
use axum::{extract::State, response::IntoResponse, Json};
use serde::Deserialize;

// ── GET /api/exitnode/platform ────────────────────────────────────────────────

pub async fn get_platform(_: State<AppState>) -> impl IntoResponse {
    Json(platform::check())
}

// ── GET /api/exitnode/deps ────────────────────────────────────────────────────

pub async fn get_deps(_: State<AppState>) -> impl IntoResponse {
    Json(deps::check_deps())
}

// ── POST /api/exitnode/deps/install ──────────────────────────────────────────

pub async fn install_deps(State(s): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    // Root check
    #[cfg(unix)]
    if !nix::unistd::getuid().is_root() {
        return Err(ApiError::ExitNode(
            "Root required to install packages".into(),
        ));
    }

    let cfg = s.config.read().await;
    let prefer_nftables = cfg.exitnode.nftables_preferred;
    drop(cfg);

    deps::install_missing(prefer_nftables)
        .map(Json)
        .map_err(ApiError::ExitNode)
}

// ── GET /api/exitnode/interfaces ──────────────────────────────────────────────

pub async fn get_interfaces(_: State<AppState>) -> Result<impl IntoResponse, ApiError> {
    interfaces::list_interfaces()
        .map(Json)
        .map_err(ApiError::ExitNode)
}

// ── GET /api/exitnode/status ──────────────────────────────────────────────────

pub async fn get_status(State(s): State<AppState>) -> impl IntoResponse {
    Json(s.exitnode_manager.status().await)
}

// ── POST /api/exitnode/enable ─────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct EnableRequest {
    pub zt_interface: String,
    pub wan_interface: String,
}

pub async fn enable(
    State(s): State<AppState>,
    Json(req): Json<EnableRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if req.zt_interface.trim().is_empty() || req.wan_interface.trim().is_empty() {
        return Err(ApiError::InvalidInput(
            "zt_interface and wan_interface are required".into(),
        ));
    }
    s.exitnode_manager
        .enable(req.zt_interface, req.wan_interface)
        .await
        .map(Json)
}

// ── POST /api/exitnode/disable ────────────────────────────────────────────────

pub async fn disable(State(s): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    s.exitnode_manager.disable().await?;
    Ok(Json(serde_json::json!({ "status": "disabled" })))
}
