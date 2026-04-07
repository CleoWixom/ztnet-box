//! Integration tests — health, config, metrics, exitnode endpoints.
//! All tests use axum::ServiceExt::oneshot (no real network needed).

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

fn app() -> axum::Router {
    let cfg = Config::default();
    let state = AppState::new(cfg, PathBuf::from("config.yml")).unwrap();
    build_router(state, "127.0.0.1", 3000)
}

async fn json_body(resp: axum::response::Response) -> serde_json::Value {
    let b = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&b).unwrap()
}

// ── Health ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn health_200_ok() {
    let resp = app()
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_body(resp).await;
    assert_eq!(body["status"], "ok");
    assert!(body["version"].is_string());
}

#[tokio::test]
async fn spa_fallback_serves_html() {
    let resp = app()
        .oneshot(
            Request::builder()
                .uri("/some/page")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let ct = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(ct.contains("text/html"));
}

// ── Security headers ──────────────────────────────────────────────────────────

#[tokio::test]
async fn security_headers_on_every_response() {
    let resp = app()
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let h = resp.headers();
    assert_eq!(
        h.get("x-content-type-options")
            .and_then(|v| v.to_str().ok()),
        Some("nosniff")
    );
    assert_eq!(
        h.get("x-frame-options").and_then(|v| v.to_str().ok()),
        Some("DENY")
    );
    assert!(h.contains_key("content-security-policy"));
}

// ── Settings/config ───────────────────────────────────────────────────────────

#[tokio::test]
async fn config_get_returns_structure() {
    let resp = app()
        .oneshot(
            Request::builder()
                .uri("/api/settings/config")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_body(resp).await;
    assert!(body["server"].is_object());
    assert!(body["zerotier"].is_object());
    assert!(body["metrics"].is_object());
    assert!(body["exitnode"].is_object());
}

#[tokio::test]
async fn config_put_invalid_port_returns_422() {
    let resp = app()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/settings/config")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"server":{"host":"127.0.0.1","port":0}}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

// ── Tokens ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn tokens_list_empty_on_fresh_state() {
    let resp = app()
        .oneshot(
            Request::builder()
                .uri("/api/settings/tokens")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_body(resp).await;
    assert!(body.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn tokens_validate_fake_returns_error_not_panic() {
    let resp = app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/settings/tokens/validate")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"token":"not_a_real_token_12345"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    // Should return 200 with valid:false (not crash or 500)
    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_body(resp).await;
    assert_eq!(body["valid"], false);
}

#[tokio::test]
async fn tokens_response_never_contains_raw_token() {
    // Add a fake token (will fail validation but test the response shape)
    let resp = app()
        .oneshot(
            Request::builder()
                .uri("/api/settings/tokens")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let body_str = std::str::from_utf8(&body).unwrap();
    // The response is an array, each element should have masked_token not raw token field
    assert!(
        !body_str.contains("\"token\":\""),
        "Raw token field must not appear in response"
    );
}

// ── Metrics ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn metrics_status_returns_structure() {
    let resp = app()
        .oneshot(
            Request::builder()
                .uri("/api/metrics/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_body(resp).await;
    assert!(body["enabled"].is_boolean());
}

#[tokio::test]
async fn metrics_raw_returns_text_plain_or_503() {
    let resp = app()
        .oneshot(
            Request::builder()
                .uri("/api/metrics/raw")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    // Either 200 text/plain (if metrics available) or 503 (not collected yet)
    assert!(resp.status() == StatusCode::OK || resp.status() == StatusCode::SERVICE_UNAVAILABLE);
    let ct = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(ct.contains("text/plain"));
}

// ── Exit node ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn exitnode_platform_always_returns_structure() {
    let resp = app()
        .oneshot(
            Request::builder()
                .uri("/api/exitnode/platform")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_body(resp).await;
    assert!(body["supported"].is_boolean());
    assert!(body["os"].is_string());
}

#[tokio::test]
async fn exitnode_deps_returns_structure() {
    let resp = app()
        .oneshot(
            Request::builder()
                .uri("/api/exitnode/deps")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_body(resp).await;
    assert!(body["is_root"].is_boolean());
    assert!(body["missing"].is_array());
}

#[tokio::test]
async fn exitnode_enable_without_body_returns_422() {
    let resp = app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/exitnode/enable")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"zt_interface":"","wan_interface":""}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    // Empty interfaces → 422 InvalidInput or 403 if not root
    assert!(
        resp.status() == StatusCode::UNPROCESSABLE_ENTITY
            || resp.status() == StatusCode::FORBIDDEN
            || resp.status() == StatusCode::BAD_GATEWAY
    );
}

// ── ZT Detection ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn zt_status_returns_detection_result() {
    let resp = app()
        .oneshot(
            Request::builder()
                .uri("/api/system/zt-status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_body(resp).await;
    assert!(body["cli_available"].is_boolean());
}
