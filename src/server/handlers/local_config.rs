//! Handlers for ZeroTier local.conf and per-network <id>.local.conf.
//! GET/PUT /api/local/config
//! GET/PUT /api/local/networks/:id/localconf

use crate::{
    server::{error::ApiError, state::AppState, validate},
    zerotier::local_config::{self, LocalConf, LocalSettings, NetworkLocalConf},
};
use axum::{
    extract::{Path, State},
    Json,
};

// ── GET /api/local/config ─────────────────────────────────────────────────────

pub async fn get_local_conf(
    State(_s): State<AppState>,
) -> Result<Json<LocalConf>, ApiError> {
    let path = local_config::local_conf_path();
    local_config::read(&path)
        .map(Json)
        .map_err(|e| ApiError::ZtLocal(e.to_string()))
}

// ── PUT /api/local/config ─────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
pub struct UpdateLocalConfRequest {
    pub settings: Option<LocalSettings>,
}

pub async fn update_local_conf(
    State(_s): State<AppState>,
    Json(req): Json<UpdateLocalConfRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let path = local_config::local_conf_path();
    let mut conf = local_config::read(&path)
        .map_err(|e| ApiError::ZtLocal(e.to_string()))?;

    if let Some(settings) = req.settings {
        // Validate before writing
        let warnings = local_config::validate_settings(&settings);
        conf.settings = Some(settings);
        local_config::write(&path, &conf)
            .map_err(|e| ApiError::ZtLocal(e.to_string()))?;
        return Ok(Json(serde_json::json!({
            "status": "ok",
            "warnings": warnings,
            "note": "Restart ZeroTier service for changes to take effect"
        })));
    }

    Ok(Json(serde_json::json!({ "status": "ok", "warnings": [] })))
}

// ── GET /api/local/networks/:id/localconf ─────────────────────────────────────

pub async fn get_network_local_conf(
    State(_s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<NetworkLocalConf>, ApiError> {
    validate::network_id(&id)?;
    local_config::read_network(&id)
        .map(Json)
        .map_err(|e| ApiError::ZtLocal(e.to_string()))
}

// ── PUT /api/local/networks/:id/localconf ─────────────────────────────────────

pub async fn update_network_local_conf(
    State(_s): State<AppState>,
    Path(id): Path<String>,
    Json(conf): Json<NetworkLocalConf>,
) -> Result<Json<serde_json::Value>, ApiError> {
    validate::network_id(&id)?;

    // Warn if allowDefault is being set without allowManaged
    let mut warnings: Vec<serde_json::Value> = Vec::new();
    if conf.allow_default == Some(true) && conf.allow_managed == Some(false) {
        warnings.push(serde_json::json!({
            "field": "allowDefault",
            "message": "allowDefault=true has no effect when allowManaged=false"
        }));
    }

    local_config::write_network(&id, &conf)
        .map_err(|e| ApiError::ZtLocal(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "status": "ok",
        "warnings": warnings,
        "note": "Changes apply immediately (no ZeroTier restart needed for network-level settings)"
    })))
}
