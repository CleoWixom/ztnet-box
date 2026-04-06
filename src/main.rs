#![allow(dead_code, unused_imports)]

mod config;
mod exitnode;
mod metrics;
mod server;
mod zerotier;

use std::sync::Arc;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use crate::{
    metrics::{cache::MetricsCache, collector::MetricsCollector},
    server::state::AppState,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ── Logging ───────────────────────────────────────────────────────────────
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("ztnet_box=info,tower_http=info")),
        )
        .init();

    // ── Config ────────────────────────────────────────────────────────────────
    let config_path = config::Config::find_config_file();
    info!(path = %config_path.display(), "loading config");

    let cfg = config::Config::load(&config_path)?;
    let host = cfg.server.host.clone();
    let port = cfg.server.port;

    // ── Metrics collector (background task) ───────────────────────────────────
    let metrics_cache = Arc::new(MetricsCache::new());

    if cfg.metrics.enabled {
        let collector = MetricsCollector::new(
            cfg.metrics.prometheus_url.clone(),
            cfg.metrics.poll_interval_seconds,
            Arc::clone(&metrics_cache),
        );
        collector.spawn();
        info!(
            url      = %cfg.metrics.prometheus_url,
            interval = cfg.metrics.poll_interval_seconds,
            "metrics collector started"
        );
    }

    // ── App state ─────────────────────────────────────────────────────────────
    let state = AppState::new_with_cache(cfg, config_path, metrics_cache)?;

    // ── Router ────────────────────────────────────────────────────────────────
    let router = server::router::build_router(state, &host, port);

    // ── Listen ────────────────────────────────────────────────────────────────
    let bind_addr = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;

    info!(
        addr    = %listener.local_addr()?,
        version = env!("CARGO_PKG_VERSION"),
        "ztnet-box listening — open http://{bind_addr} in your browser"
    );

    axum::serve(listener, router).await?;
    Ok(())
}
