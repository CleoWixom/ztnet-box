use crate::{
    server::{error::ApiError, state::AppState},
    zerotier::detection,
};
use axum::{extract::State, http::StatusCode, Json};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tempfile;
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

// ── Generate Moon ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct GenerateMoonRequest {
    /// Stable endpoint for the moon, e.g. "1.2.3.4/9993"
    pub stable_endpoint: String,
    /// Optional: existing ZT identity for this node (if empty, uses /var/lib/zerotier-one/identity.secret)
    pub identity_path: Option<String>,
}

#[derive(Serialize)]
pub struct GenerateMoonResponse {
    pub world_id: String,
    pub moon_file_base64: String,
    pub instructions: String,
}

pub async fn generate_moon(
    State(_): State<AppState>,
    Json(body): Json<GenerateMoonRequest>,
) -> Result<Json<GenerateMoonResponse>, ApiError> {
    if body.stable_endpoint.trim().is_empty() {
        return Err(ApiError::InvalidInput(
            "stable_endpoint is required (e.g. '1.2.3.4/9993')".into(),
        ));
    }

    // Find zerotier-idtool
    let idtool = which_idtool().ok_or_else(|| {
        ApiError::Internal(
            "zerotier-idtool not found — install ZeroTier on this machine".into(),
        )
    })?;

    // Read identity.public from the identity path
    let identity_path = body
        .identity_path
        .unwrap_or_else(|| "/var/lib/zerotier-one/identity.public".into());

    let identity = std::fs::read_to_string(&identity_path).map_err(|e| {
        ApiError::Internal(format!("Cannot read identity from {identity_path}: {e}"))
    })?;

    // Write identity to a temp dir so zerotier-idtool can use it
    let tmpdir = tempfile::tempdir()
        .map_err(|e| ApiError::Internal(format!("tempdir: {e}")))?;
    let id_file = tmpdir.path().join("identity.public");
    std::fs::write(&id_file, identity.trim())
        .map_err(|e| ApiError::Internal(format!("write identity: {e}")))?;

    // Run: zerotier-idtool genmoon <identity.public>
    let out = std::process::Command::new(&idtool)
        .args(["genmoon", id_file.to_str().unwrap()])
        .current_dir(tmpdir.path())
        .output()
        .map_err(|e| ApiError::Internal(format!("zerotier-idtool failed: {e}")))?;

    if !out.status.success() {
        return Err(ApiError::Internal(format!(
            "zerotier-idtool genmoon failed: {}",
            String::from_utf8_lossy(&out.stderr)
        )));
    }

    // Find generated .moon file
    let moon_file = std::fs::read_dir(tmpdir.path())
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .filter_map(|e| e.ok())
        .find(|e| {
            e.path()
                .extension()
                .map(|x| x == "moon")
                .unwrap_or(false)
        })
        .ok_or_else(|| ApiError::Internal("genmoon did not produce a .moon file".into()))?;

    let moon_bytes = std::fs::read(moon_file.path())
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    // Extract world ID from filename (e.g. "000000deadbeef.moon")
    let world_id = moon_file
        .path()
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.trim_start_matches('0').to_string())
        .unwrap_or_default();

    let moon_b64 = B64.encode(&moon_bytes);

    tracing::info!(world_id = %world_id, endpoint = %body.stable_endpoint, "moon generated");

    Ok(Json(GenerateMoonResponse {
        world_id: world_id.clone(),
        moon_file_base64: moon_b64,
        instructions: format!(
            "1. Copy the .moon file to /var/lib/zerotier-one/moons.d/ on your root server.\n\
             2. Restart zerotier-one on the root server.\n\
             3. Orbit this moon on client nodes: zerotier-cli orbit {world_id} {world_id}\n\
             4. Or use the 'Orbit a Moon' form on this page with World ID: {world_id}"
        ),
    }))
}

fn which_idtool() -> Option<std::path::PathBuf> {
    for candidate in &[
        "/usr/sbin/zerotier-idtool",
        "/usr/local/sbin/zerotier-idtool",
        "/usr/bin/zerotier-idtool",
    ] {
        let p = std::path::Path::new(candidate);
        if p.exists() {
            return Some(p.to_path_buf());
        }
    }
    // Try PATH
    std::process::Command::new("which")
        .arg("zerotier-idtool")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            String::from_utf8(o.stdout)
                .ok()
                .map(|s| std::path::PathBuf::from(s.trim()))
        })
}
