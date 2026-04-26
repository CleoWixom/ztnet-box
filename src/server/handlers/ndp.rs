use crate::{
    exitnode::ndp::{self, NdpConfig},
    server::{error::ApiError, state::AppState, validate},
};
use axum::{extract::State, response::IntoResponse, Json};
use serde::Deserialize;
use tracing;

// ── GET /api/exitnode/ndp/status ──────────────────────────────────────────────

pub async fn get_status(_: State<AppState>) -> impl IntoResponse {
    Json(ndp::check_status())
}

// ── POST /api/exitnode/ndp/install ────────────────────────────────────────────

pub async fn install(_: State<AppState>) -> Result<impl IntoResponse, ApiError> {
    #[cfg(unix)]
    if !nix::unistd::getuid().is_root() {
        return Err(ApiError::ZtLocal("Root required to install ndppd".into()));
    }
    ndp::install()
        .map(Json)
        .map_err(|e| ApiError::ZtLocal(e.to_string()))
}

// ── POST /api/exitnode/ndp/enable ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct EnableRequest {
    /// WAN interface name (e.g. "eth0")
    pub wan_iface: String,
    /// ZeroTier interface prefix (default "zt")
    #[serde(default = "default_zt_prefix")]
    pub zt_prefix: String,
    /// IPv6 prefix to proxy NDP for (e.g. "2001:db8::/64")
    pub ipv6_prefix: String,
}

fn default_zt_prefix() -> String {
    "zt".into()
}

pub async fn enable(
    State(_s): State<AppState>,
    Json(req): Json<EnableRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if req.wan_iface.trim().is_empty() {
        return Err(ApiError::InvalidInput("wan_iface is required".into()));
    }
    validate::cidr(&req.ipv6_prefix)?;

    #[cfg(unix)]
    if !nix::unistd::getuid().is_root() {
        return Err(ApiError::ZtLocal(
            "Root privileges required to configure ndppd".into(),
        ));
    }

    let cfg = NdpConfig {
        wan_iface: req.wan_iface,
        zt_prefix: req.zt_prefix,
        ipv6_prefix: req.ipv6_prefix,
    };

    tracing::info!("enabling NDP proxy");
    ndp::enable(&cfg)
        .map(Json)
        .map_err(|e| ApiError::ZtLocal(e.to_string()))
}

// ── POST /api/exitnode/ndp/disable ────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct DisableRequest {
    /// If true, remove /etc/ndppd.conf as well
    #[serde(default)]
    pub remove_config: bool,
}

pub async fn disable(
    State(_s): State<AppState>,
    Json(req): Json<DisableRequest>,
) -> Result<impl IntoResponse, ApiError> {
    #[cfg(unix)]
    if !nix::unistd::getuid().is_root() {
        return Err(ApiError::ZtLocal("Root privileges required".into()));
    }

    tracing::info!("disabling NDP proxy");
    ndp::disable(req.remove_config)
        .map(Json)
        .map_err(|e| ApiError::ZtLocal(e.to_string()))
}
