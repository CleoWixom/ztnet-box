#![allow(dead_code, unused_imports)]
mod config;
mod exitnode;
mod metrics;
mod server;
mod zerotier;

use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Инициализация логирования
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("ztnet_box=info,tower_http=debug")),
        )
        .init();

    let config_path = config::Config::find_config_file();
    info!(path = %config_path.display(), "Loading configuration");

    let cfg = config::Config::load(&config_path)?;

    info!(
        host    = %cfg.server.host,
        port    = cfg.server.port,
        version = env!("CARGO_PKG_VERSION"),
        "Starting ztnet-box"
    );

    let bind_addr = format!("{}:{}", cfg.server.host, cfg.server.port);
    let state = server::state::AppState::new(cfg, config_path)?;
    let router = server::router::build_router(state);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!(addr = %listener.local_addr()?, "Listening — open http://{bind_addr} in your browser");

    axum::serve(listener, router).await?;
    Ok(())
}
