use crate::{
    relay::{deploy, LocalRelayConfig, RelayDeployConfig, RelayStatus, RemoteRelayInfo},
    server::{error::ApiError, state::AppState},
    zerotier::local_config::{self, LocalSettings},
};
use axum::{extract::State, response::IntoResponse, Json};
use serde::Deserialize;
use tokio::sync::RwLock;

// ── GET /api/relay/status ─────────────────────────────────────────────────────

pub async fn get_status(State(s): State<AppState>) -> impl IntoResponse {
    let local = read_local_relay();
    let remote = s.relay_remote.read().await.clone();
    Json(RelayStatus { local, remote })
}

// ── PUT /api/relay/local ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct UpdateLocalRequest {
    pub force_tcp_relay: Option<bool>,
    /// "ip/port" or null to clear
    pub tcp_fallback_relay: Option<String>,
}

pub async fn update_local(
    State(_s): State<AppState>,
    Json(req): Json<UpdateLocalRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if let Some(ref ep) = req.tcp_fallback_relay {
        validate_relay_endpoint(ep)?;
    }

    let path = local_config::local_conf_path();
    let mut conf = local_config::read(&path).unwrap_or_default();

    // Ensure settings exists
    let settings = conf.settings.get_or_insert_with(LocalSettings::default);
    if let Some(v) = req.force_tcp_relay {
        settings.force_tcp_relay = Some(v);
    }
    settings.tcp_fallback_relay = req.tcp_fallback_relay.clone();

    let warnings = local_config::validate_settings(settings);
    local_config::write(&path, &conf).map_err(|e| ApiError::ZtLocal(e.to_string()))?;

    tracing::info!(
        force = ?req.force_tcp_relay,
        fallback = ?req.tcp_fallback_relay,
        "relay local config updated"
    );

    let (force, fallback) = conf
        .settings
        .as_ref()
        .map(|s| (s.force_tcp_relay, s.tcp_fallback_relay.clone()))
        .unwrap_or((None, None));

    Ok(Json(serde_json::json!({
        "applied": {
            "force_tcp_relay": force,
            "tcp_fallback_relay": fallback,
        },
        "warnings": warnings.iter().map(|w| &w.message).collect::<Vec<_>>(),
    })))
}

// ── POST /api/relay/deploy ────────────────────────────────────────────────────

pub async fn deploy_relay(
    State(s): State<AppState>,
    Json(cfg): Json<RelayDeployConfig>,
) -> Result<impl IntoResponse, ApiError> {
    if cfg.host.trim().is_empty() {
        return Err(ApiError::InvalidInput("host is required".into()));
    }
    if cfg.password.is_none() && cfg.key_path.is_none() {
        return Err(ApiError::InvalidInput(
            "either password or key_path is required".into(),
        ));
    }

    let info = tokio::task::spawn_blocking(move || deploy::deploy(&cfg))
        .await
        .map_err(|e| ApiError::ZtLocal(format!("task join: {e}")))?
        .map_err(|e| ApiError::ZtLocal(e.to_string()))?;

    *s.relay_remote.write().await = Some(info.clone());

    // Auto-configure local.conf to point at the newly deployed relay
    let relay_endpoint = format!("{}/{}", info.host, info.port);
    let path = local_config::local_conf_path();
    if let Ok(mut conf) = local_config::read(&path) {
        conf.settings
            .get_or_insert_with(LocalSettings::default)
            .tcp_fallback_relay = Some(relay_endpoint);
        let _ = local_config::write(&path, &conf);
    }

    Ok(Json(serde_json::json!({
        "status": "deployed",
        "relay": info,
    })))
}

// ── GET /api/relay/verify ─────────────────────────────────────────────────────

pub async fn verify_relay(State(s): State<AppState>) -> impl IntoResponse {
    let remote = s.relay_remote.read().await.clone();
    match remote {
        None => Json(serde_json::json!({ "reachable": false, "reason": "no relay deployed" })),
        Some(mut info) => {
            let ok = tokio::task::spawn_blocking({
                let h = info.host.clone();
                let p = info.port;
                move || deploy::verify(&h, p)
            })
            .await
            .unwrap_or(false);

            info.reachable = Some(ok);
            *s.relay_remote.write().await = Some(info.clone());
            Json(serde_json::json!({
                "reachable": ok,
                "host": info.host,
                "port": info.port,
            }))
        }
    }
}

// ── DELETE /api/relay/remote ──────────────────────────────────────────────────

pub async fn remove_relay(
    State(s): State<AppState>,
    Json(cfg): Json<RelayDeployConfig>,
) -> Result<impl IntoResponse, ApiError> {
    let info = s.relay_remote.read().await.clone();
    if let Some(ref remote) = info {
        let remote_clone = remote.clone();
        tokio::task::spawn_blocking(move || deploy::remove(&remote_clone, &cfg))
            .await
            .map_err(|e| ApiError::ZtLocal(format!("task join: {e}")))?
            .map_err(|e| ApiError::ZtLocal(e.to_string()))?;
    }
    *s.relay_remote.write().await = None;

    // Clear local.conf relay endpoint
    let path = local_config::local_conf_path();
    if let Ok(mut conf) = local_config::read(&path) {
        if let Some(ref mut s) = conf.settings {
            s.tcp_fallback_relay = None;
        }
        let _ = local_config::write(&path, &conf);
    }

    Ok(Json(serde_json::json!({ "status": "removed" })))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn read_local_relay() -> LocalRelayConfig {
    let path = local_config::local_conf_path();
    local_config::read(&path)
        .map(|c| {
            let s = c.settings.unwrap_or_default();
            LocalRelayConfig {
                force_tcp_relay: s.force_tcp_relay.unwrap_or(false),
                tcp_fallback_relay: s.tcp_fallback_relay,
            }
        })
        .unwrap_or_default()
}

/// Validates "ip/port" relay endpoint format.
fn validate_relay_endpoint(ep: &str) -> Result<(), ApiError> {
    let parts: Vec<&str> = ep.splitn(2, '/').collect();
    if parts.len() != 2 {
        return Err(ApiError::InvalidInput(format!(
            "relay endpoint must be 'ip/port', got '{ep}'"
        )));
    }
    let port: u16 = parts[1]
        .parse()
        .map_err(|_| ApiError::InvalidInput(format!("invalid port in relay endpoint '{ep}'")))?;
    if port == 0 {
        return Err(ApiError::InvalidInput("port must be > 0".into()));
    }
    Ok(())
}

// ── Shared state type alias (used in state.rs) ────────────────────────────────

pub type RelayRemoteState = RwLock<Option<RemoteRelayInfo>>;
