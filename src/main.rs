use std::sync::Arc;
use tracing::{info, warn};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use ztnet_box::{
    config,
    metrics::{cache::MetricsCache, collector::MetricsCollector},
    server::{
        self,
        log_collector::{CollectorLayer, LogCollector},
    },
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let log_collector = LogCollector::new();

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("ztnet_box=info,tower_http=info")),
        )
        .with(CollectorLayer::new(log_collector.clone()))
        .init();

    let config_path = config::Config::find_config_file();
    info!(path = %config_path.display(), "loading config");

    let cfg = config::Config::load(&config_path)?;
    let host = cfg.server.host.clone();
    let port = cfg.server.port;

    let metrics_cache = Arc::new(MetricsCache::new());

    if cfg.metrics.enabled {
        MetricsCollector::start(
            cfg.metrics.prometheus_url.clone(),
            cfg.metrics.poll_interval_seconds,
            Arc::clone(&metrics_cache),
            cfg.metrics.metricstoken_file.clone(),
        );
        info!(
            url        = %cfg.metrics.prometheus_url,
            interval   = cfg.metrics.poll_interval_seconds,
            token_file = %cfg.metrics.metricstoken_file.display(),
            "metrics collector started"
        );
    }

    if host != "127.0.0.1" && host != "::1" && host != "localhost" {
        warn!(
            host = %host,
            "SECURITY: server bound to non-loopback address — ensure network-level access control"
        );
    }

    let state = server::state::AppState::new_with_cache_and_collector(
        cfg,
        config_path,
        metrics_cache,
        log_collector,
    )?;
    let router = server::router::build_router(state, &host, port);
    let bind = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&bind).await?;

    info!(
        addr    = %listener.local_addr()?,
        version = env!("CARGO_PKG_VERSION"),
        "ztnet-box ready — open http://{bind}"
    );

    axum::serve(listener, router).await?;
    Ok(())
}
