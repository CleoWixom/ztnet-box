use crate::server::{log_collector::LogLevel, state::AppState};
use axum::{
    extract::{Query, State},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Json,
    },
};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};

// ── GET /api/logs ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct LogQuery {
    /// Minimum level to include (error | warn | info | debug | trace)
    pub level: Option<String>,
    /// Max entries to return (default 200, max 500)
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    200
}

pub async fn get_logs(State(s): State<AppState>, Query(q): Query<LogQuery>) -> impl IntoResponse {
    let min_level = q.level.as_deref().and_then(|l| l.parse::<LogLevel>().ok());
    let limit = q.limit.min(500);
    let entries = s.log_collector.entries(min_level, limit);
    Json(entries)
}

// ── GET /api/logs/stream  (SSE) ───────────────────────────────────────────────

pub async fn stream_logs(
    State(s): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let rx = s.log_collector.subscribe();

    let stream = BroadcastStream::new(rx).filter_map(|result| {
        match result {
            Ok(entry) => {
                // Serialize to JSON; skip on error
                serde_json::to_string(&entry)
                    .ok()
                    .map(|json| Ok(Event::default().event("log").data(json)))
            }
            // Lagged — receiver missed some messages; skip silently
            Err(_) => None,
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

// ── PUT /api/logs/level ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SetLevelRequest {
    pub level: String,
}

#[derive(Debug, Serialize)]
pub struct LevelResponse {
    pub level: String,
}

pub async fn set_level(
    State(s): State<AppState>,
    Json(req): Json<SetLevelRequest>,
) -> Result<impl IntoResponse, axum::http::StatusCode> {
    let level = req
        .level
        .parse::<LogLevel>()
        .map_err(|_| axum::http::StatusCode::UNPROCESSABLE_ENTITY)?;
    s.log_collector.set_level(level);
    Ok(Json(LevelResponse {
        level: level.to_string(),
    }))
}

// ── GET /api/logs/level ───────────────────────────────────────────────────────

pub async fn get_level(State(s): State<AppState>) -> impl IntoResponse {
    Json(LevelResponse {
        level: s.log_collector.current_level().to_string(),
    })
}

// ── DELETE /api/logs ──────────────────────────────────────────────────────────

pub async fn clear_logs(State(s): State<AppState>) -> impl IntoResponse {
    s.log_collector.clear();
    Json(serde_json::json!({ "cleared": true }))
}
