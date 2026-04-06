use axum::{
    body::Body,
    http::{Request, Response},
    middleware::Next,
};
use std::time::Instant;
use tracing::info;

/// Логирует каждый запрос: METHOD /path → STATUS latency_ms
pub async fn log_request(req: Request<Body>, next: Next) -> Response<Body> {
    let method = req.method().clone();
    let path = req.uri().path().to_owned();
    let start = Instant::now();

    let resp = next.run(req).await;
    let status = resp.status().as_u16();
    let ms = start.elapsed().as_millis();

    info!(method = %method, path = %path, status, latency_ms = ms, "←");
    resp
}
