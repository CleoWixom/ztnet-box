use axum::{
    body::Body,
    http::{Request, Response},
    middleware::Next,
};
use std::time::Instant;

/// Логирует каждый запрос: METHOD /path → STATUS latency_ms
/// Уровень debug — не попадает в LogCollector (собирает от info и выше)
/// и не отображается в Log Panel по умолчанию.
pub async fn log_request(req: Request<Body>, next: Next) -> Response<Body> {
    let method = req.method().clone();
    let path = req.uri().path().to_owned();
    let start = Instant::now();

    let resp = next.run(req).await;
    let status = resp.status().as_u16();
    let ms = start.elapsed().as_millis();

    // Use warn for 4xx/5xx errors so they still appear in Log Panel
    if status >= 500 {
        tracing::warn!(method = %method, path = %path, status, latency_ms = ms, "← HTTP error");
    } else if status >= 400 {
        tracing::warn!(method = %method, path = %path, status, latency_ms = ms, "← HTTP client error");
    } else {
        tracing::debug!(method = %method, path = %path, status, latency_ms = ms, "←");
    }
    resp
}
