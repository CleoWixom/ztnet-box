use super::{middleware::log_request, state::AppState};
use axum::{
    http::{HeaderName, HeaderValue, Method},
    middleware,
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
use serde_json::json;
use tower_http::{
    cors::{Any, CorsLayer},
    set_header::SetResponseHeaderLayer,
};

static INDEX_HTML: &str = include_str!("../../www/build/index.html");

pub fn build_router(state: AppState) -> Router {
    // CORS — только localhost
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any)
        .allow_origin([
            "http://127.0.0.1:3000".parse().unwrap(),
            "http://localhost:3000".parse().unwrap(),
        ]);

    let api = Router::new()
        .route("/health", get(health_handler))
        .route("/system/zt-status", get(zt_status_handler))
        .route(
            "/system/zt-install",
            axum::routing::post(zt_install_handler),
        );

    Router::new()
        // Фронтенд
        .route("/", get(index_handler))
        // API
        .nest("/api", api)
        // Фоллбэк для SPA
        .fallback(get(spa_fallback))
        // Middleware
        .layer(middleware::from_fn(log_request))
        .layer(cors)
        // Security headers
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

async fn zt_status_handler() -> impl IntoResponse {
    let result = crate::zerotier::detection::detect();
    Json(result)
}

async fn zt_install_handler() -> impl IntoResponse {
    let pm = crate::zerotier::detection::detect_package_manager();
    match pm {
        Some(pm) => match crate::zerotier::detection::install(pm) {
            Ok(result) => Json(json!(result)).into_response(),
            Err(e) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string(), "code": "ERR_INSTALL" })),
            )
                .into_response(),
        },
        None => (
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "status": "unsupported_platform",
                "reason": "No supported package manager found (apt, dnf, pacman, brew)"
            })),
        )
            .into_response(),
    }
}
