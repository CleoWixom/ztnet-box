//! Integration tests against the real ZeroTier Central API.
//!
//! These tests require a valid Central API token in the environment:
//!   ZT_CENTRAL_TOKEN=<your_token> cargo test --test api_central
//!
//! Tests are SKIPPED (not failed) when ZT_CENTRAL_TOKEN is not set.
//! They create and immediately delete any resources they touch.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use std::path::PathBuf;
use tower::ServiceExt;
use ztnet_box::{
    config::Config,
    server::{router::build_router, state::AppState},
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn central_token() -> Option<String> {
    std::env::var("ZT_CENTRAL_TOKEN")
        .ok()
        .filter(|s| !s.trim().is_empty())
}

/// Build app configured with the Central token and pointing to real Central API.
/// Returns None to skip if token not available.
async fn app_or_skip() -> Option<(axum::Router, String)> {
    let token = central_token()?;

    // Verify token is actually valid before running tests
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .ok()?;
    let ok = client
        .get("https://api.zerotier.com/api/v1/status")
        .bearer_auth(&token)
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false);

    if !ok {
        eprintln!("[SKIP] ZT_CENTRAL_TOKEN is set but Central API returned non-200");
        return None;
    }

    let mut cfg = Config::default();
    cfg.zerotier.central.base_url = "https://api.zerotier.com/api/v1".into();

    use ztnet_box::config::schema::{CentralToken, RateLimit};
    let ct = CentralToken::new("test-token".into(), token.clone(), RateLimit::Free);
    cfg.zerotier.central.tokens = vec![ct.clone()];
    cfg.zerotier.central.active_token_id = ct.id.clone();

    let state = AppState::new(cfg, PathBuf::from("config.yml")).ok()?;
    let app = build_router(state, "127.0.0.1", 3000);
    Some((app, token))
}

async fn body_json(r: axum::response::Response) -> serde_json::Value {
    let b = r.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&b).unwrap_or(serde_json::Value::Null)
}

async fn get(app: &axum::Router, uri: &str) -> axum::response::Response {
    app.clone()
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap()
}

async fn post_json(app: &axum::Router, uri: &str, body: &str) -> axum::response::Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn put_json(app: &axum::Router, uri: &str, body: &str) -> axum::response::Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn delete(app: &axum::Router, uri: &str) -> axum::response::Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
}

// ── Central status + user ─────────────────────────────────────────────────────

#[tokio::test]
async fn central_status_returns_structure() {
    let Some((app, _)) = app_or_skip().await else {
        return;
    };

    let r = get(&app, "/api/central/status").await;
    assert_eq!(r.status(), StatusCode::OK);
    let body = body_json(r).await;
    assert!(body["online"].is_boolean());
    assert!(body["version"].is_string());
}

#[tokio::test]
async fn central_user_returns_account_info() {
    let Some((app, _)) = app_or_skip().await else {
        return;
    };

    let r = get(&app, "/api/central/user").await;
    assert_eq!(r.status(), StatusCode::OK);
    let body = body_json(r).await;
    assert!(
        body["id"].is_string() || body["email"].is_string(),
        "user response must contain id or email"
    );
}

// ── Central tokens (stored in app state, no Central call) ─────────────────────

#[tokio::test]
async fn settings_tokens_list_includes_injected_token() {
    let Some((app, _)) = app_or_skip().await else {
        return;
    };

    let r = get(&app, "/api/settings/tokens").await;
    assert_eq!(r.status(), StatusCode::OK);
    let body = body_json(r).await;
    let tokens = body.as_array().expect("must be array");
    assert!(
        !tokens.is_empty(),
        "token list must contain at least the injected token"
    );

    let token = &tokens[0];
    assert!(token["id"].is_string());
    assert!(token["name"].is_string());
    assert!(token["masked_token"].is_string());
    assert!(token["is_active"].is_boolean());
    // Raw token must never appear
    assert!(
        !token.to_string().contains(r#""token":"#),
        "raw token must never be in response"
    );
}

#[tokio::test]
async fn settings_token_add_invalid_token_returns_error() {
    let Some((app, _)) = app_or_skip().await else {
        return;
    };

    let r = post_json(
        &app,
        "/api/settings/tokens",
        r#"{"name":"bad","token":"completely_invalid_token_xyz"}"#,
    )
    .await;
    // Must fail validation — not 201
    assert_ne!(
        r.status(),
        StatusCode::CREATED,
        "invalid token must not be added successfully"
    );
    // Must be a client or gateway error, not a panic
    assert!(r.status().is_client_error() || r.status() == StatusCode::BAD_GATEWAY);
}

#[tokio::test]
async fn settings_token_empty_name_returns_422() {
    let Some((app, _)) = app_or_skip().await else {
        return;
    };

    let r = post_json(
        &app,
        "/api/settings/tokens",
        r#"{"name":"","token":"sometoken"}"#,
    )
    .await;
    assert_eq!(r.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn settings_token_validate_real_token() {
    let Some((app, token)) = app_or_skip().await else {
        return;
    };

    let body = serde_json::json!({"token": token}).to_string();
    let r = post_json(&app, "/api/settings/tokens/validate", &body).await;
    assert_eq!(r.status(), StatusCode::OK);
    let resp = body_json(r).await;
    assert_eq!(resp["valid"], true, "real token must validate as valid");
    assert!(
        resp["account_status"].is_object(),
        "must include account_status"
    );
}

// ── Central networks CRUD ─────────────────────────────────────────────────────

#[tokio::test]
async fn central_network_list_returns_array() {
    let Some((app, _)) = app_or_skip().await else {
        return;
    };

    let r = get(&app, "/api/central/networks").await;
    assert_eq!(r.status(), StatusCode::OK);
    let body = body_json(r).await;
    assert!(body.is_array(), "central networks must be an array");
}

#[tokio::test]
async fn central_network_crud() {
    let Some((app, _)) = app_or_skip().await else {
        return;
    };

    // Create
    let r = post_json(
        &app,
        "/api/central/networks",
        r#"{"config":{"name":"ztnet-integration-test"}}"#,
    )
    .await;
    assert!(
        r.status() == StatusCode::OK || r.status() == StatusCode::CREATED,
        "create central network: got {}",
        r.status()
    );
    let net = body_json(r).await;
    let net_id = match net["id"].as_str() {
        Some(id) => id.to_string(),
        None => {
            eprintln!("[SKIP] No network id returned");
            return;
        }
    };

    // Get
    let r = get(&app, &format!("/api/central/networks/{net_id}")).await;
    assert_eq!(r.status(), StatusCode::OK);
    let got = body_json(r).await;
    assert_eq!(got["id"].as_str().unwrap(), net_id);

    // Update — rename
    let r = put_json(
        &app,
        &format!("/api/central/networks/{net_id}"),
        r#"{"config":{"name":"ztnet-integration-renamed"}}"#,
    )
    .await;
    assert_eq!(r.status(), StatusCode::OK, "update central network failed");
    let updated = body_json(r).await;
    assert_eq!(
        updated["config"]["name"].as_str().unwrap_or(""),
        "ztnet-integration-renamed"
    );

    // List members (empty on fresh network)
    let r = get(&app, &format!("/api/central/networks/{net_id}/members")).await;
    assert_eq!(r.status(), StatusCode::OK);
    assert!(body_json(r).await.is_array());

    // Delete — cleanup
    let r = delete(&app, &format!("/api/central/networks/{net_id}")).await;
    assert!(
        r.status() == StatusCode::OK || r.status() == StatusCode::NO_CONTENT,
        "delete central network: got {}",
        r.status()
    );

    // Confirm deleted
    let r = get(&app, &format!("/api/central/networks/{net_id}")).await;
    assert!(
        r.status() == StatusCode::NOT_FOUND || r.status() == StatusCode::BAD_GATEWAY,
        "deleted network must return 404, got {}",
        r.status()
    );
}
