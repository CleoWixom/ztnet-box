use crate::{
    server::{error::ApiError, state::AppState},
    zerotier::detection,
};
use axum::{extract::State, Json};

pub async fn zt_status(
    State(_state): State<AppState>,
) -> Result<Json<detection::ZtDetectionResult>, ApiError> {
    Ok(Json(detection::detect()))
}

pub async fn zt_install(
    State(_state): State<AppState>,
) -> Result<Json<detection::InstallResult>, ApiError> {
    match detection::detect_package_manager() {
        Some(pm) => detection::install(pm)
            .map(Json)
            .map_err(|e| ApiError::Internal(e.to_string())),
        None => Ok(Json(detection::InstallResult::UnsupportedPlatform {
            reason: "No supported package manager found (apt, dnf, pacman, brew)".into(),
        })),
    }
}
