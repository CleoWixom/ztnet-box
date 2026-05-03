use crate::{
    server::{error::ApiError, state::AppState},
    zerotier::detection,
};
use axum::{extract::State, http::StatusCode, Json};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing;

pub async fn zt_status(
    State(_state): State<AppState>,
) -> Result<Json<detection::ZtDetectionResult>, ApiError> {
    Ok(Json(detection::detect()))
}

pub async fn zt_install(
    State(_state): State<AppState>,
) -> Result<Json<detection::InstallResult>, ApiError> {
    tracing::info!("ZeroTier install requested");
    match detection::detect_package_manager() {
        Some(pm) => detection::install(pm)
            .map(Json)
            .map_err(|e| ApiError::Internal(e.to_string())),
        None => Ok(Json(detection::InstallResult::UnsupportedPlatform {
            reason: "No supported package manager found (apt, dnf, pacman, brew)".into(),
        })),
    }
}

// ── Planet file ───────────────────────────────────────────────────────────────

const PLANET_PATH: &str = "/var/lib/zerotier-one/planet";
const PLANET_BACKUP: &str = "/var/lib/zerotier-one/planet.default";

#[derive(Serialize)]
pub struct PlanetFileInfo {
    pub is_custom: bool,
    pub base64: Option<String>,
    /// UTC milliseconds of last modification, or null
    pub modified_ms: Option<u64>,
}

#[derive(Deserialize)]
pub struct PlanetFileUpload {
    pub base64: String,
}

pub async fn get_planet_file(
    State(_): State<AppState>,
) -> Result<Json<PlanetFileInfo>, ApiError> {
    let path = PathBuf::from(PLANET_PATH);
    if !path.exists() {
        return Ok(Json(PlanetFileInfo {
            is_custom: false,
            base64: None,
            modified_ms: None,
        }));
    }
    let bytes = std::fs::read(&path).map_err(|e| ApiError::Internal(e.to_string()))?;
    let modified_ms = path
        .metadata()
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| {
            t.duration_since(std::time::UNIX_EPOCH)
                .ok()
                .map(|d| d.as_millis() as u64)
        });
    // Determine if custom: compare size to backup (if backup exists)
    let is_custom = PathBuf::from(PLANET_BACKUP).exists();
    Ok(Json(PlanetFileInfo {
        is_custom,
        base64: Some(B64.encode(&bytes)),
        modified_ms,
    }))
}

pub async fn upload_planet_file(
    State(_): State<AppState>,
    Json(body): Json<PlanetFileUpload>,
) -> Result<StatusCode, ApiError> {
    if body.base64.is_empty() {
        return Err(ApiError::InvalidInput("base64 must not be empty".into()));
    }
    let bytes = B64
        .decode(&body.base64)
        .map_err(|e| ApiError::InvalidInput(format!("invalid base64: {e}")))?;

    // Backup default planet if no backup exists yet
    let planet = PathBuf::from(PLANET_PATH);
    let backup = PathBuf::from(PLANET_BACKUP);
    if planet.exists() && !backup.exists() {
        std::fs::copy(&planet, &backup).map_err(|e| ApiError::Internal(e.to_string()))?;
    }

    tracing::info!(bytes = bytes.len(), "uploading custom planet file");
    std::fs::write(&planet, &bytes).map_err(|e| ApiError::Internal(e.to_string()))?;

    // Signal zerotier-one to reload (best-effort; requires running daemon + SIGHUP support)
    let _ = std::process::Command::new("pkill")
        .args(["-HUP", "zerotier-one"])
        .status();

    tracing::info!("custom planet file written; sent SIGHUP to zerotier-one");
    Ok(StatusCode::NO_CONTENT)
}

pub async fn reset_planet_file(
    State(_): State<AppState>,
) -> Result<StatusCode, ApiError> {
    let planet = PathBuf::from(PLANET_PATH);
    let backup = PathBuf::from(PLANET_BACKUP);

    if backup.exists() {
        tracing::info!("restoring default planet file from backup");
        std::fs::copy(&backup, &planet).map_err(|e| ApiError::Internal(e.to_string()))?;
        std::fs::remove_file(&backup).ok();
    } else {
        // No backup — just remove the file so ZeroTier uses its built-in default
        tracing::info!("removing custom planet file (no backup — ZT will use compiled-in default)");
        std::fs::remove_file(&planet).ok();
    }

    let _ = std::process::Command::new("pkill")
        .args(["-HUP", "zerotier-one"])
        .status();

    Ok(StatusCode::NO_CONTENT)
}
