use super::{
    handlers::{config as cfg_handler, system as sys_handler},
    middleware::log_request,
    state::AppState,
};
use axum::{
    http::{HeaderName, HeaderValue, Method},
    middleware,
    response::{Html, IntoResponse},
    routing::{get, post, put},
    Json, Router,
};
use serde_json::json;
use tower_http::{cors::CorsLayer, set_header::SetResponseHeaderLayer};

static INDEX_HTML: &str = include_str!("../../www/build/index.html");

pub fn build_router(state: AppState, host: &str, port: u16) -> Router {
    let origin_host = format!("http://{host}:{port}")
        .parse::<HeaderValue>()
        .expect("valid origin");
    let origin_lo = format!("http://localhost:{port}")
        .parse::<HeaderValue>()
        .expect("valid origin");

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(tower_http::cors::Any)
        .allow_origin([origin_host, origin_lo]);

    let api = Router::new()
        .route("/health", get(health_handler))
        .route("/system/zt-status", get(sys_handler::zt_status))
        .route("/system/zt-install", post(sys_handler::zt_install))
        .route("/settings/config", get(cfg_handler::get_config))
        .route("/settings/config", put(cfg_handler::update_config));

    Router::new()
        .route("/", get(index_handler))
        .nest("/api", api)
        .fallback(get(spa_fallback))
        .layer(middleware::from_fn(log_request))
        .layer(cors)
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("content-security-policy"),
            HeaderValue::from_static(
                "default-src 'self'; \
                 script-src 'self' 'unsafe-inline'; \
                 style-src 'self' 'unsafe-inline'",
            ),
        ))
        .with_state(state)
}

async fn index_handler() -> Html<&'static str> {
    Html(INDEX_HTML)
}
async fn spa_fallback() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn health_handler() -> impl IntoResponse {
    Json(json!({ "status": "ok", "version": env!("CARGO_PKG_VERSION") }))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use std::path::PathBuf;
    use tower::ServiceExt;

    fn test_router() -> Router {
        let cfg = crate::config::Config::default();
        let state = AppState::new(cfg, PathBuf::from("config.yml")).unwrap();
        build_router(state, "127.0.0.1", 3000)
    }

    #[tokio::test]
    async fn health_returns_200() {
        let app = test_router();
        let req = Request::builder()
            .uri("/api/health")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn health_body_contains_ok() {
        let app = test_router();
        let req = Request::builder()
            .uri("/api/health")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "ok");
        assert!(json["version"].is_string());
    }

    #[tokio::test]
    async fn index_returns_html() {
        let app = test_router();
        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(ct.contains("text/html"), "expected text/html, got {ct}");
    }

    #[tokio::test]
    async fn spa_fallback_returns_html() {
        let app = test_router();
        let req = Request::builder()
            .uri("/some/deep/route")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn security_headers_present() {
        let app = test_router();
        let req = Request::builder()
            .uri("/api/health")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        let hdrs = resp.headers();
        assert_eq!(
            hdrs.get("x-content-type-options")
                .and_then(|v| v.to_str().ok()),
            Some("nosniff")
        );
        assert_eq!(
            hdrs.get("x-frame-options").and_then(|v| v.to_str().ok()),
            Some("DENY")
        );
        assert!(hdrs.contains_key("content-security-policy"));
    }

    #[tokio::test]
    async fn config_endpoint_returns_200() {
        let app = test_router();
        let req = Request::builder()
            .uri("/api/settings/config")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn zt_status_endpoint_returns_200() {
        let app = test_router();
        let req = Request::builder()
            .uri("/api/system/zt-status")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
