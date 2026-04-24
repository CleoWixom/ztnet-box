//! Integration tests against a real ZeroTier daemon (127.0.0.1:9993).
//!
//! These tests require:
//!   - zerotier-one daemon running on the test host
//!   - /var/lib/zerotier-one/authtoken.secret readable (or ZT_TOKEN env var)
//!
//! They are automatically SKIPPED (not failed) when the daemon is unreachable.
//! In CI run them with:
//!   sudo ZT_RUNNING=1 cargo test --test api_local
//!
//! The test suite is designed to be fully idempotent — every created resource
//! is cleaned up in the same test, even on failure (best-effort).

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

fn zt_local_token() -> Option<String> {
    if let Ok(t) = std::env::var("ZT_TOKEN") {
        if !t.trim().is_empty() {
            return Some(t.trim().to_string());
        }
    }
    std::fs::read_to_string("/var/lib/zerotier-one/authtoken.secret")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

async fn app_or_skip() -> Option<axum::Router> {
    let token = zt_local_token()?;

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .ok()?;

    let ok = client
        .get("http://127.0.0.1:9993/status")
        .header("X-ZT1-Auth", &token)
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false);

    if !ok {
        eprintln!("[SKIP] ZeroTier daemon not reachable at 127.0.0.1:9993");
        return None;
    }

    let mut cfg = Config::default();
    cfg.zerotier.local.api_url = "http://127.0.0.1:9993".into();
    let tmp = tempfile::NamedTempFile::new().ok()?;
    std::fs::write(tmp.path(), &token).ok()?;
    cfg.zerotier.local.token_file = tmp.path().to_path_buf();
    std::mem::forget(tmp);

    let state = AppState::new(cfg, PathBuf::from("config.yml")).ok()?;
    Some(build_router(state, "127.0.0.1", 3000))
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

// ── Node status ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn local_node_status_returns_address() {
    let Some(app) = app_or_skip().await else {
        return;
    };

    let r = get(&app, "/api/local/status").await;
    assert_eq!(r.status(), StatusCode::OK);
    let body = body_json(r).await;
    assert!(body["address"].is_string(), "address field must be present");
    assert_eq!(
        body["address"].as_str().unwrap().len(),
        10,
        "ZT address is 10 hex chars"
    );
    assert!(body["online"].is_boolean());
    assert!(body["version"].is_string());
}

// ── Peers ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn local_peers_returns_array() {
    let Some(app) = app_or_skip().await else {
        return;
    };

    let r = get(&app, "/api/local/peers").await;
    assert_eq!(r.status(), StatusCode::OK);
    let body = body_json(r).await;
    assert!(body.is_array());
    for peer in body.as_array().unwrap() {
        assert!(peer["address"].is_string());
        assert!(peer["role"].is_string());
        assert!(peer["latency"].is_number());
    }
}

#[tokio::test]
async fn local_peer_invalid_id_returns_422() {
    let Some(app) = app_or_skip().await else {
        return;
    };

    let r = get(&app, "/api/local/peers/ZZZZZZZZZZ").await;
    assert_eq!(r.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

// ── Networks — join / list / leave ────────────────────────────────────────────

#[tokio::test]
async fn local_networks_list_returns_array() {
    let Some(app) = app_or_skip().await else {
        return;
    };

    let r = get(&app, "/api/local/networks").await;
    assert_eq!(r.status(), StatusCode::OK);
    assert!(body_json(r).await.is_array());
}

#[tokio::test]
async fn local_network_join_and_leave() {
    let Some(app) = app_or_skip().await else {
        return;
    };

    let test_net =
        std::env::var("ZT_TEST_NETWORK").unwrap_or_else(|_| "8056c2e21c000001".to_string());

    // Join
    let r = post_json(&app, &format!("/api/local/networks/{test_net}"), "{}").await;
    assert!(
        r.status() == StatusCode::OK || r.status() == StatusCode::CREATED,
        "join should return 200/201, got {}",
        r.status()
    );

    // Verify it appears in list
    let r = get(&app, "/api/local/networks").await;
    let nets = body_json(r).await;
    let found = nets
        .as_array()
        .unwrap()
        .iter()
        .any(|n| n["id"].as_str() == Some(&test_net));
    assert!(found, "joined network must appear in /api/local/networks");

    // Get individual network
    let r = get(&app, &format!("/api/local/networks/{test_net}")).await;
    assert_eq!(r.status(), StatusCode::OK);
    let body = body_json(r).await;
    assert_eq!(body["id"].as_str().unwrap_or(""), test_net);

    // Leave
    let r = delete(&app, &format!("/api/local/networks/{test_net}")).await;
    assert!(
        r.status() == StatusCode::OK || r.status() == StatusCode::NO_CONTENT,
        "leave should return 200/204, got {}",
        r.status()
    );

    // Verify gone
    let r = get(&app, "/api/local/networks").await;
    let nets = body_json(r).await;
    let still_there = nets
        .as_array()
        .unwrap()
        .iter()
        .any(|n| n["id"].as_str() == Some(&test_net));
    assert!(!still_there, "network must be gone after leave");
}

// ── Controller — network CRUD ─────────────────────────────────────────────────

#[tokio::test]
async fn controller_network_crud() {
    let Some(app) = app_or_skip().await else {
        return;
    };

    let r = get(&app, "/api/local/status").await;
    let status = body_json(r).await;
    let address = match status["address"].as_str() {
        Some(a) => a.to_string(),
        None => {
            eprintln!("[SKIP] Could not get node address");
            return;
        }
    };

    // Create controller network
    let r = post_json(
        &app,
        "/api/local/controller/networks",
        r#"{"name":"ztnet-test-crud","private":true}"#,
    )
    .await;
    if r.status() == StatusCode::BAD_GATEWAY {
        eprintln!("[SKIP] Local controller not available (ZT not running as controller)");
        return;
    }
    assert!(
        r.status() == StatusCode::OK || r.status() == StatusCode::CREATED,
        "create network: got {}",
        r.status()
    );
    let net = body_json(r).await;
    let net_id = net["id"]
        .as_str()
        .expect("created network must have id")
        .to_string();
    assert_eq!(net_id.len(), 16, "network ID must be 16 chars");
    assert!(
        net_id.starts_with(&address),
        "network ID must start with node address"
    );

    // Get network
    let r = get(&app, &format!("/api/local/controller/networks/{net_id}")).await;
    assert_eq!(r.status(), StatusCode::OK);
    let got = body_json(r).await;
    assert_eq!(got["id"].as_str().unwrap(), net_id);

    // List networks — must include ours
    let r = get(&app, "/api/local/controller/networks").await;
    assert_eq!(r.status(), StatusCode::OK);
    let list = body_json(r).await;
    let ids: Vec<&str> = list
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert!(
        ids.contains(&net_id.as_str()),
        "created network must appear in list"
    );

    // Update — rename
    let r = put_json(
        &app,
        &format!("/api/local/controller/networks/{net_id}"),
        r#"{"name":"ztnet-test-renamed"}"#,
    )
    .await;
    assert_eq!(r.status(), StatusCode::OK, "update network failed");
    let updated = body_json(r).await;
    assert_eq!(updated["name"].as_str().unwrap_or(""), "ztnet-test-renamed");

    // Delete
    let r = delete(&app, &format!("/api/local/controller/networks/{net_id}")).await;
    assert!(
        r.status() == StatusCode::OK || r.status() == StatusCode::NO_CONTENT,
        "delete network: got {}",
        r.status()
    );

    // Confirm gone
    let r = get(&app, &format!("/api/local/controller/networks/{net_id}")).await;
    assert!(
        r.status() == StatusCode::NOT_FOUND || r.status() == StatusCode::BAD_GATEWAY,
        "deleted network must return 404"
    );
}

// ── Controller — members CRUD ─────────────────────────────────────────────────

#[tokio::test]
async fn controller_members_crud() {
    let Some(app) = app_or_skip().await else {
        return;
    };

    let r = get(&app, "/api/local/status").await;
    let status = body_json(r).await;
    let address = match status["address"].as_str() {
        Some(a) => a.to_string(),
        None => {
            eprintln!("[SKIP] No node address");
            return;
        }
    };

    // Create a network to add members to
    let r = post_json(
        &app,
        "/api/local/controller/networks",
        r#"{"name":"ztnet-test-members","private":true}"#,
    )
    .await;
    if r.status() == StatusCode::BAD_GATEWAY {
        eprintln!("[SKIP] Controller not available");
        return;
    }
    let net = body_json(r).await;
    let net_id = net["id"].as_str().unwrap().to_string();

    let member_id = &address;
    let member_path = format!("/api/local/controller/networks/{net_id}/members/{member_id}");

    // Add/authorize member via PUT
    let r = put_json(
        &app,
        &member_path,
        r#"{"authorized":true,"activeBridge":false}"#,
    )
    .await;
    assert_eq!(r.status(), StatusCode::OK, "authorize member failed");
    let member = body_json(r).await;
    assert_eq!(member["authorized"].as_bool(), Some(true));

    // List members
    let r = get(
        &app,
        &format!("/api/local/controller/networks/{net_id}/members"),
    )
    .await;
    assert_eq!(r.status(), StatusCode::OK);
    let members = body_json(r).await;
    assert!(
        members.is_array() || members.is_object(),
        "members must be array or object"
    );

    // Get single member
    let r = get(&app, &member_path).await;
    assert_eq!(r.status(), StatusCode::OK);

    // Update member — deauthorize
    let r = put_json(&app, &member_path, r#"{"authorized":false}"#).await;
    assert_eq!(r.status(), StatusCode::OK);
    let updated = body_json(r).await;
    assert_eq!(updated["authorized"].as_bool(), Some(false));

    // Delete member
    let r = delete(&app, &member_path).await;
    assert!(
        r.status() == StatusCode::OK || r.status() == StatusCode::NO_CONTENT,
        "delete member: got {}",
        r.status()
    );

    // Cleanup: delete the test network
    delete(&app, &format!("/api/local/controller/networks/{net_id}")).await;
}

// ── Moons ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn local_moons_list_returns_array() {
    let Some(app) = app_or_skip().await else {
        return;
    };

    let r = get(&app, "/api/local/moons").await;
    assert_eq!(r.status(), StatusCode::OK);
    assert!(body_json(r).await.is_array());
}

// ── Local config (ZT node settings) ──────────────────────────────────────────

#[tokio::test]
async fn local_config_roundtrip() {
    let Some(app) = app_or_skip().await else {
        return;
    };

    let r = get(&app, "/api/local/config").await;
    assert_eq!(r.status(), StatusCode::OK);
    let current = body_json(r).await;
    assert!(current.is_object(), "local config must be an object");

    let r = put_json(
        &app,
        "/api/local/config",
        &serde_json::to_string(&current).unwrap(),
    )
    .await;
    assert!(
        r.status() == StatusCode::OK || r.status() == StatusCode::BAD_GATEWAY,
        "local config PUT: got {}",
        r.status()
    );
}
