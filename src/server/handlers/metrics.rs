use crate::server::{error::ApiError, state::AppState};
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Serialize;

// ── GET /api/metrics ──────────────────────────────────────────────────────────

pub async fn get_metrics(State(s): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    match s.metrics_cache.snapshot().await {
        Some(snap) => Ok(Json(snap).into_response()),
        None => Err(ApiError::ZtLocal("Metrics not yet available".into())),
    }
}

// ── GET /api/metrics/raw ──────────────────────────────────────────────────────

pub async fn get_raw(State(s): State<AppState>) -> impl IntoResponse {
    match s.metrics_cache.raw_text().await {
        Some(text) => (
            StatusCode::OK,
            [(
                header::CONTENT_TYPE,
                "text/plain; version=0.0.4; charset=utf-8",
            )],
            text,
        )
            .into_response(),
        None => (
            StatusCode::SERVICE_UNAVAILABLE,
            [(header::CONTENT_TYPE, "text/plain")],
            "# Metrics not yet collected\n".to_string(),
        )
            .into_response(),
    }
}

// ── GET /api/metrics/status ───────────────────────────────────────────────────

#[derive(Serialize)]
pub struct MetricsStatus {
    pub enabled: bool,
    pub last_updated: Option<chrono::DateTime<chrono::Utc>>,
    pub error: Option<String>,
}

pub async fn get_status(State(s): State<AppState>) -> impl IntoResponse {
    let cfg = s.config.read().await;
    let enabled = cfg.metrics.enabled;
    drop(cfg);

    Json(MetricsStatus {
        enabled,
        last_updated: s.metrics_cache.last_updated().await,
        error: s.metrics_cache.last_error().await,
    })
}
