use crate::{
    exitnode::{deps, interfaces, platform},
    server::{error::ApiError, state::AppState, validate},
    zerotier::local_config,
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
    /// ZeroTier network_id — used to check allowDefault in <id>.local.conf
    pub network_id: Option<String>,
    /// Enable IPv6 ip6tables rules on this exit node
    #[serde(default)]
    pub enable_ipv6: bool,
    /// Optional IPv6 prefix to scope the FORWARD rules (e.g. "2001:db8::/64")
    pub ipv6_prefix: Option<String>,
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

    // Validate optional IPv6 prefix
    if let Some(ref pfx) = req.ipv6_prefix {
        validate::cidr(pfx).map_err(|_| {
            ApiError::InvalidInput(format!("ipv6_prefix is not a valid CIDR: {pfx}"))
        })?;
    }

    // allowDefault / allowGlobal conflict check
    let mut warnings: Vec<String> = Vec::new();
    if let Some(ref net_id) = req.network_id {
        if validate::network_id(net_id).is_ok() {
            match local_config::read_network(net_id) {
                Ok(nc) => {
                    if nc.allow_default != Some(true) {
                        warnings.push(format!(
                            "allowDefault is not set for network {net_id}. \
                             ZeroTier clients using this exit node must set allowDefault=1 \
                             in their network settings to route all traffic through this gateway."
                        ));
                    }
                    if nc.allow_global != Some(true) {
                        warnings.push(format!(
                            "allowGlobal is not set for network {net_id}. \
                             IPv6 routing through the exit node requires allowGlobal=1."
                        ));
                    }
                }
                Err(_) => {
                    warnings.push(format!(
                        "No local.conf found for network {net_id}. \
                         Clients may need allowDefault=1 to use this exit node."
                    ));
                }
            }
        }
    }

    // Extra IPv6 warnings
    if req.enable_ipv6 {
        if req.network_id.is_none() {
            warnings.push(
                "IPv6 enabled without a network_id — cannot verify allowGlobal/allowDefault. \
                 Ensure ZeroTier clients have both flags set."
                    .into(),
            );
        }
        warnings.push(
            "IPv6 NAT (ip6tables MASQUERADE) is enabled. \
             For native IPv6 delegation without NAT, configure ndppd and assign a public prefix."
                .into(),
        );
    }

    let state = s
        .exitnode_manager
        .enable(
            req.zt_interface,
            req.wan_interface,
            req.enable_ipv6,
            req.ipv6_prefix,
            req.network_id,
        )
        .await?;

    Ok(Json(serde_json::json!({
        "enabled":       state.enabled,
        "zt_interface":  state.zt_interface,
        "zt_network_id": state.zt_network_id,
        "wan_interface": state.wan_interface,
        "backend":       state.backend,
        "enable_ipv6":   state.enable_ipv6,
        "ipv6_prefix":   state.ipv6_prefix,
        "applied_at":    state.applied_at,
        "warnings":      warnings,
    })))
}

// ── POST /api/exitnode/disable ────────────────────────────────────────────────

pub async fn disable(State(s): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    s.exitnode_manager.disable().await?;
    Ok(Json(serde_json::json!({ "status": "disabled" })))
}
