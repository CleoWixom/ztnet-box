use super::{
    handlers::{
        central as central_handler, config as cfg_handler, local as local_handler,
        system as sys_handler, tokens as tok_handler,
    },
    middleware::log_request,
    state::AppState,
};
use axum::{
    http::{HeaderName, HeaderValue, Method},
    middleware,
    response::{Html, IntoResponse},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde_json::json;
use tower_http::{cors::CorsLayer, set_header::SetResponseHeaderLayer};

static INDEX_HTML: &str = include_str!("../../www/build/index.html");

pub fn build_router(state: AppState, host: &str, port: u16) -> Router {
    let origin_host = format!("http://{host}:{port}")
        .parse::<HeaderValue>()
        .expect("origin");
    let origin_lo = format!("http://localhost:{port}")
        .parse::<HeaderValue>()
        .expect("origin");

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(tower_http::cors::Any)
        .allow_origin([origin_host, origin_lo]);

    // /api/local/*
    let local = Router::new()
        .route("/status", get(local_handler::node_status))
        .route("/networks", get(local_handler::list_networks))
        .route(
            "/networks/:id",
            get(local_handler::get_network)
                .post(local_handler::join_network)
                .delete(local_handler::leave_network),
        )
        .route("/peers", get(local_handler::list_peers))
        .route("/peers/:id", get(local_handler::get_peer))
        .route(
            "/controller/networks",
            get(local_handler::list_controller_networks)
                .post(local_handler::create_controller_network),
        )
        .route(
            "/controller/networks/:id",
            get(local_handler::get_controller_network)
                .put(local_handler::update_controller_network)
                .delete(local_handler::delete_controller_network),
        )
        .route(
            "/controller/networks/:id/members",
            get(local_handler::list_members),
        )
        .route(
            "/controller/networks/:id/members/:node_id",
            get(local_handler::get_member)
                .put(local_handler::update_member)
                .delete(local_handler::delete_member),
        )
        .route("/moons", get(local_handler::list_moons))
        .route(
            "/moons/:world_id",
            post(local_handler::orbit_moon).delete(local_handler::deorbit_moon),
        );

    // /api/central/*
    let central = Router::new()
        .route(
            "/networks",
            get(central_handler::list_networks).post(central_handler::create_network),
        )
        .route(
            "/networks/:id",
            get(central_handler::get_network)
                .put(central_handler::update_network)
                .delete(central_handler::delete_network),
        )
        .route("/networks/:id/members", get(central_handler::list_members))
        .route(
            "/networks/:id/members/:node_id",
            get(central_handler::get_member)
                .put(central_handler::update_member)
                .delete(central_handler::delete_member),
        )
        .route("/user", get(central_handler::get_user))
        .route("/status", get(central_handler::get_status));

    // /api/settings/tokens/*
    let tokens = Router::new()
        .route(
            "/",
            get(tok_handler::list_tokens).post(tok_handler::add_token),
        )
        .route("/validate", post(tok_handler::validate_token))
        .route(
            "/:id",
            put(tok_handler::update_token).delete(tok_handler::delete_token),
        )
        .route("/:id/activate", post(tok_handler::activate_token));

    let api = Router::new()
        .route("/health", get(health_handler))
        .route("/system/zt-status", get(sys_handler::zt_status))
        .route("/system/zt-install", post(sys_handler::zt_install))
        .route(
            "/settings/config",
            get(cfg_handler::get_config).put(cfg_handler::update_config),
        )
        .nest("/settings/tokens", tokens)
        .nest("/local", local)
        .nest("/central", central);

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
    Json(json!({ "status": "ok", "version": env!("CARGO_PKG_VERSION") }))
}

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
        let resp = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn health_body_contains_ok() {
        let resp = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["status"], "ok");
        assert!(v["version"].is_string());
    }

    #[tokio::test]
    async fn index_returns_html() {
        let resp = test_router()
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
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

    #[tokio::test]
    async fn spa_fallback_serves_html() {
        let resp = test_router()
            .oneshot(
                Request::builder()
                    .uri("/deep/route")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn security_headers_present() {
        let resp = test_router()
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

    #[tokio::test]
    async fn central_no_token_returns_502() {
        let resp = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/central/networks")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        // No token configured → ApiError::ZtCentral → 502
        assert_eq!(resp.status(), StatusCode::BAD_GATEWAY);
    }
}
