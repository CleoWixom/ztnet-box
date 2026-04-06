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
use tower_http::{
    cors::{Any, CorsLayer},
    set_header::SetResponseHeaderLayer,
};

static INDEX_HTML: &str = include_str!("../../www/build/index.html");

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any)
        .allow_origin([
            "http://127.0.0.1:3000".parse().unwrap(),
            "http://localhost:3000".parse().unwrap(),
        ]);

    let api = Router::new()
        // Health
        .route("/health", get(health_handler))
        // System / ZeroTier detection
        .route("/system/zt-status", get(sys_handler::zt_status))
        .route("/system/zt-install", post(sys_handler::zt_install))
        // Settings / Config
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
                "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'",
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
    Json(json!({
        "status":  "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}
