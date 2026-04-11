//! Physical Network Routing handlers.
//! Docs: https://docs.zerotier.com/route-between-phys-and-virt/

use crate::{
    physnet::{self, conflicts, deps, PhysNetConfig, PhysNetState},
    server::{error::ApiError, state::AppState, validate},
};
use axum::{extract::State, response::IntoResponse, Json};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

// ── Shared state accessor ─────────────────────────────────────────────────────

pub type PhysNetStateArc = Arc<RwLock<PhysNetState>>;

// ── GET /api/physnet/platform ─────────────────────────────────────────────────

pub async fn get_platform(_: State<AppState>) -> impl IntoResponse {
    #[derive(serde::Serialize)]
    struct Platform {
        supported: bool,
        os: &'static str,
        reason: Option<&'static str>,
    }

    #[cfg(target_os = "linux")]
    return Json(Platform {
        supported: true,
        os: "linux",
        reason: None,
    });

    #[cfg(not(target_os = "linux"))]
    return Json(Platform {
        supported: false,
        os: std::env::consts::OS,
        reason: Some("Physical Network Routing requires Linux (iptables)"),
    });
}

// ── GET /api/physnet/deps ─────────────────────────────────────────────────────

pub async fn get_deps(_: State<AppState>) -> impl IntoResponse {
    Json(deps::check())
}

// ── GET /api/physnet/status ───────────────────────────────────────────────────

pub async fn get_status(State(s): State<AppState>) -> impl IntoResponse {
    Json(s.physnet_state.read().await.clone())
}

// ── POST /api/physnet/enable ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct EnableRequest {
    pub zt_iface: String,
    pub phy_iface: String,
    pub phy_subnet: String,
    pub zt_addr: String,
    pub network_id: String,
}

pub async fn enable(
    State(s): State<AppState>,
    Json(req): Json<EnableRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate inputs
    if req.zt_iface.trim().is_empty() {
        return Err(ApiError::InvalidInput("zt_iface required".into()));
    }
    if req.phy_iface.trim().is_empty() {
        return Err(ApiError::InvalidInput("phy_iface required".into()));
    }
    validate::cidr(&req.phy_subnet)?;
    validate::ip_addr(&req.zt_addr)?;
    validate::network_id(&req.network_id)?;

    // Root check
    #[cfg(unix)]
    if !nix::unistd::getuid().is_root() {
        return Err(ApiError::ZtLocal(
            "Root privileges required for iptables rules".into(),
        ));
    }

    // Conflict check
    let exitnode_on = s.exitnode_manager.status().await.enabled;
    let bridge_on = s.bridge_state.read().await.enabled;
    let report = conflicts::check(&req.phy_subnet, None, exitnode_on, bridge_on);
    let has_error = report
        .conflicts
        .iter()
        .any(|c| matches!(c.severity, conflicts::ConflictSeverity::Error));
    if has_error {
        let msgs: Vec<_> = report
            .conflicts
            .iter()
            .map(|c| c.message.as_str())
            .collect();
        return Err(ApiError::ZtLocal(msgs.join("; ")));
    }

    let cfg = PhysNetConfig {
        zt_iface: req.zt_iface,
        phy_iface: req.phy_iface,
        phy_subnet: req.phy_subnet.clone(),
        zt_addr: req.zt_addr,
        network_id: req.network_id,
    };

    physnet::rules::apply(&cfg).map_err(|e| ApiError::ZtLocal(e.to_string()))?;

    let state = PhysNetState {
        enabled: true,
        config: Some(cfg),
        applied_at: Some(chrono::Utc::now()),
    };
    *s.physnet_state.write().await = state.clone();

    let route_hint = physnet::managed_route_hint(&req.phy_subnet);
    Ok(Json(serde_json::json!({
        "status": "enabled",
        "state": state,
        "warnings": report.conflicts.iter()
            .filter(|c| matches!(c.severity, conflicts::ConflictSeverity::Warning))
            .map(|c| &c.message)
            .collect::<Vec<_>>(),
        "next_step": format!(
            "Add managed route in ZeroTier Central: destination={route_hint} via=<zt_addr>"
        ),
    })))
}

// ── POST /api/physnet/disable ─────────────────────────────────────────────────

pub async fn disable(State(s): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    #[cfg(unix)]
    if !nix::unistd::getuid().is_root() {
        return Err(ApiError::ZtLocal("Root privileges required".into()));
    }

    let st = s.physnet_state.read().await.clone();
    if let Some(cfg) = st.config {
        physnet::rules::remove(&cfg).map_err(|e| ApiError::ZtLocal(e.to_string()))?;
    }
    *s.physnet_state.write().await = PhysNetState::default();
    Ok(Json(serde_json::json!({ "status": "disabled" })))
}
