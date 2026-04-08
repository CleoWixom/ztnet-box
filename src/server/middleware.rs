use axum::{
    body::Body,
    http::{Request, Response},
    middleware::Next,
};
use std::time::Instant;
use tracing::{info, warn};

/// Logs each request: METHOD /path → STATUS latency_ms
pub async fn log_request(req: Request<Body>, next: Next) -> Response<Body> {
    let method = req.method().clone();
    let path = req.uri().path().to_owned();
    let start = Instant::now();
    let resp = next.run(req).await;
    let ms = start.elapsed().as_millis();
    info!(method = %method, path = %path, status = resp.status().as_u16(), latency_ms = ms, "←");
    resp
}

/// Emits a warning if the server is bound to a non-localhost address.
pub fn warn_if_public_bind(host: &str) {
    if host != "127.0.0.1" && host != "::1" && host != "localhost" {
        warn!(
            host = %host,
            "SECURITY: Server is bound to a public address. \
             ZeroBox has no authentication — ensure network-level access control \
             (firewall, VPN, etc.) is in place."
        );
    }
}
