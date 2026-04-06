use axum::{
    body::Body,
    http::{Request, Response},
    middleware::Next,
};
use std::time::Instant;
use tracing::info;

/// Логирование запросов: method path → status latency
pub async fn log_request(req: Request<Body>, next: Next) -> Response<Body> {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let start = Instant::now();

    let resp = next.run(req).await;

    let latency = start.elapsed();
    let status = resp.status().as_u16();

    info!(
        method = %method,
        path   = %path,
        status = status,
        latency_ms = latency.as_millis(),
        "request"
    );

    resp
}
