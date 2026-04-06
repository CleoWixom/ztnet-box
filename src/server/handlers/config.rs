use crate::{
    config::schema::{
        CentralConfig, Config, ExitNodeConfig, MetricsConfig, ServerConfig, ZeroTierConfig,
    },
    server::{error::ApiError, state::AppState},
};
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

// ── GET /api/settings/config ──────────────────────────────────────────────────

/// Config view — токены маскируются (первые 4 символа + ***)
#[derive(Debug, Serialize)]
pub struct ConfigView {
    pub server: ServerConfig,
    pub zerotier: ZeroTierConfigView,
    pub metrics: MetricsConfig,
    pub exitnode: ExitNodeConfig,
}

#[derive(Debug, Serialize)]
pub struct ZeroTierConfigView {
    pub local: crate::config::schema::LocalConfig,
    pub central: CentralConfigView,
}

#[derive(Debug, Serialize)]
pub struct CentralConfigView {
    pub base_url: String,
    pub tokens: Vec<TokenView>,
    pub active_token_id: String,
}

#[derive(Debug, Serialize)]
pub struct TokenView {
    pub id: String,
    pub name: String,
    pub token: String, // masked
    pub rate_limit: crate::config::schema::RateLimit,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn get_config(State(state): State<AppState>) -> Result<Json<ConfigView>, ApiError> {
    let cfg = state.config.read().await;

    let tokens = cfg
        .zerotier
        .central
        .tokens
        .iter()
        .map(|t| TokenView {
            id: t.id.clone(),
            name: t.name.clone(),
            token: t.masked_token(),
            rate_limit: t.rate_limit.clone(),
            created_at: t.created_at,
        })
        .collect();

    Ok(Json(ConfigView {
        server: cfg.server.clone(),
        zerotier: ZeroTierConfigView {
            local: cfg.zerotier.local.clone(),
            central: CentralConfigView {
                base_url: cfg.zerotier.central.base_url.clone(),
                tokens,
                active_token_id: cfg.zerotier.central.active_token_id.clone(),
            },
        },
        metrics: cfg.metrics.clone(),
        exitnode: cfg.exitnode.clone(),
    }))
}

// ── PUT /api/settings/config ──────────────────────────────────────────────────

/// Обновляемые секции конфига (токены управляются отдельно)
#[derive(Debug, Deserialize)]
pub struct UpdateConfigRequest {
    pub server: Option<ServerConfig>,
    pub zerotier_local: Option<crate::config::schema::LocalConfig>,
    pub metrics: Option<MetricsConfig>,
    pub exitnode: Option<ExitNodeConfig>,
    pub central_base_url: Option<String>,
}

pub async fn update_config(
    State(state): State<AppState>,
    Json(req): Json<UpdateConfigRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Validate server port if provided
    if let Some(ref s) = req.server {
        if s.port == 0 {
            return Err(ApiError::InvalidInput("port must be 1–65535".into()));
        }
    }

    {
        let mut cfg = state.config.write().await;

        if let Some(s) = req.server {
            cfg.server = s;
        }
        if let Some(l) = req.zerotier_local {
            cfg.zerotier.local = l;
        }
        if let Some(m) = req.metrics {
            cfg.metrics = m;
        }
        if let Some(e) = req.exitnode {
            cfg.exitnode = e;
        }
        if let Some(url) = req.central_base_url {
            cfg.zerotier.central.base_url = url;
        }

        cfg.save(&state.config_path)
            .map_err(|e| ApiError::Config(e.to_string()))?;
    }

    Ok(Json(serde_json::json!({ "status": "ok" })))
}
