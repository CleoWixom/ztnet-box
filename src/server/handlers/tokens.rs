use crate::{
    config::schema::{CentralToken, RateLimit},
    server::{error::ApiError, state::AppState},
    zerotier::central::{client::ZtCentralClient, types::AccountStatus},
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

// ── View types (masked token, never send raw) ─────────────────────────────────

#[derive(Debug, Serialize)]
pub struct TokenView {
    pub id: String,
    pub name: String,
    pub masked_token: String,
    pub rate_limit: RateLimit,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub is_active: bool,
}

fn to_view(t: &CentralToken, active_id: &str) -> TokenView {
    TokenView {
        id: t.id.clone(),
        name: t.name.clone(),
        masked_token: t.masked_token(),
        rate_limit: t.rate_limit.clone(),
        created_at: t.created_at,
        is_active: t.id == active_id,
    }
}

// ── GET /api/settings/tokens ──────────────────────────────────────────────────

pub async fn list_tokens(State(s): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let tokens = s.token_store.list().await;
    let active = s.token_store.active_token().await;
    let active_id = active.as_ref().map(|t| t.id.as_str()).unwrap_or("");
    let views: Vec<TokenView> = tokens.iter().map(|t| to_view(t, active_id)).collect();
    Ok(Json(views))
}

// ── POST /api/settings/tokens ─────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct AddTokenRequest {
    pub name: String,
    pub token: String,
}

pub async fn add_token(
    State(s): State<AppState>,
    Json(req): Json<AddTokenRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if req.name.trim().is_empty() {
        return Err(ApiError::InvalidInput("name must not be empty".into()));
    }
    if req.token.trim().is_empty() {
        return Err(ApiError::InvalidInput("token must not be empty".into()));
    }

    // Validate and detect rate_limit
    let cfg = s.config.read().await;
    let base_url = cfg.zerotier.central.base_url.clone();
    drop(cfg);

    let rate_limit = validate_and_detect_rate(&base_url, &req.token).await?;

    let t = s.token_store.add(req.name, req.token, rate_limit).await;

    // Persist to config
    persist_tokens(&s).await?;

    let active_id = s
        .token_store
        .active_token()
        .await
        .map(|a| a.id)
        .unwrap_or_default();
    Ok((StatusCode::CREATED, Json(to_view(&t, &active_id))))
}

// ── PUT /api/settings/tokens/:id ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct UpdateTokenRequest {
    pub name: Option<String>,
    pub token: Option<String>,
}

pub async fn update_token(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateTokenRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let existing = s.token_store.find(&id).await.ok_or(ApiError::NotFound)?;

    let new_name = req.name.unwrap_or(existing.name);
    let new_token = req.token.unwrap_or_else(|| existing.token.clone());

    // Re-validate only if the token value actually changed
    let rate_limit = if new_token != existing.token {
        let cfg = s.config.read().await;
        let base_url = cfg.zerotier.central.base_url.clone();
        drop(cfg);
        validate_and_detect_rate(&base_url, &new_token).await?
    } else {
        existing.rate_limit.clone()
    };

    // Update in-place — preserves the original UUID and position in the list
    let t = s
        .token_store
        .update(&id, new_name, new_token, rate_limit)
        .await
        .ok_or(ApiError::NotFound)?;

    persist_tokens(&s).await?;

    let active_id = s
        .token_store
        .active_token()
        .await
        .map(|a| a.id)
        .unwrap_or_default();
    Ok(Json(to_view(&t, &active_id)))
}

// ── DELETE /api/settings/tokens/:id ──────────────────────────────────────────

pub async fn delete_token(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let removed = s.token_store.remove(&id).await;
    if !removed {
        return Err(ApiError::NotFound);
    }
    persist_tokens(&s).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ── POST /api/settings/tokens/:id/activate ───────────────────────────────────

pub async fn activate_token(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let ok = s.token_store.set_active(&id).await;
    if !ok {
        return Err(ApiError::NotFound);
    }
    persist_tokens(&s).await?;
    Ok(Json(serde_json::json!({ "is_active": true, "id": id })))
}

// ── POST /api/settings/tokens/validate ───────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ValidateRequest {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct ValidateResponse {
    pub valid: bool,
    pub account_status: Option<AccountStatus>,
    pub rate_limit: Option<RateLimit>,
    pub error: Option<String>,
}

pub async fn validate_token(
    State(s): State<AppState>,
    Json(req): Json<ValidateRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let cfg = s.config.read().await;
    let base_url = cfg.zerotier.central.base_url.clone();
    drop(cfg);

    match probe_token(&base_url, &req.token).await {
        Ok(status) => {
            let rl = status.rate_limit();
            Ok(Json(ValidateResponse {
                valid: true,
                account_status: Some(status),
                rate_limit: Some(rl),
                error: None,
            }))
        }
        Err(e) => Ok(Json(ValidateResponse {
            valid: false,
            account_status: None,
            rate_limit: None,
            error: Some(e.to_string()),
        })),
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

async fn probe_token(base_url: &str, token: &str) -> Result<AccountStatus, ApiError> {
    let client = ZtCentralClient::new(base_url.to_string(), token.to_string(), &RateLimit::Free);
    client.account_status().await
}

async fn validate_and_detect_rate(base_url: &str, token: &str) -> Result<RateLimit, ApiError> {
    let status = probe_token(base_url, token).await?;
    Ok(status.rate_limit())
}

async fn persist_tokens(s: &AppState) -> Result<(), ApiError> {
    let tokens = s.token_store.list().await;
    let active_id = s
        .token_store
        .active_token()
        .await
        .map(|t| t.id)
        .unwrap_or_default();
    let mut cfg = s.config.write().await;
    cfg.zerotier.central.tokens = tokens;
    cfg.zerotier.central.active_token_id = active_id;
    cfg.save(&s.config_path)
        .map_err(|e| ApiError::Config(e.to_string()))?;
    Ok(())
}
