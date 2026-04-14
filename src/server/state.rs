use crate::{
    bridge::BridgeState,
    config::{schema::Config, ConfigError},
    exitnode::ExitNodeManager,
    metrics::cache::MetricsCache,
    physnet::PhysNetState,
    relay::RemoteRelayInfo,
    runtime_state,
    server::log_collector::LogCollector,
    zerotier::central::token_store::TokenStore,
};
use std::{path::PathBuf, sync::Arc};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<RwLock<Config>>,
    pub config_path: PathBuf,
    pub token_store: Arc<TokenStore>,
    pub metrics_cache: Arc<MetricsCache>,
    pub exitnode_manager: Arc<ExitNodeManager>,
    pub physnet_state: Arc<RwLock<PhysNetState>>,
    pub bridge_state: Arc<RwLock<BridgeState>>,
    pub relay_remote: Arc<RwLock<Option<RemoteRelayInfo>>>,
    pub log_collector: LogCollector,
    /// Path where bridge/physnet/relay state is persisted across restarts.
    pub runtime_state_path: PathBuf,
}

impl AppState {
    pub fn new(config: Config, config_path: PathBuf) -> Result<Self, ConfigError> {
        Self::new_with_cache(config, config_path, Arc::new(MetricsCache::new()))
    }

    pub fn new_with_cache(
        config: Config,
        config_path: PathBuf,
        metrics_cache: Arc<MetricsCache>,
    ) -> Result<Self, ConfigError> {
        Self::new_with_cache_and_collector(config, config_path, metrics_cache, LogCollector::new())
    }

    pub fn new_with_cache_and_collector(
        config: Config,
        config_path: PathBuf,
        metrics_cache: Arc<MetricsCache>,
        log_collector: LogCollector,
    ) -> Result<Self, ConfigError> {
        let tokens = config.zerotier.central.tokens.clone();
        let active_token_id = config.zerotier.central.active_token_id.clone();
        let base_url = config.zerotier.central.base_url.clone();
        let exitnode_cfg = config.exitnode.clone();

        let token_store =
            Arc::new(TokenStore::new(tokens, active_token_id).with_base_url(base_url));

        // Load persisted bridge/physnet/relay state from the last run
        let runtime_state_path = runtime_state::state_path();
        let saved = runtime_state::load(&runtime_state_path);
        tracing::debug!(path = %runtime_state_path.display(), "loaded runtime state");

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            config_path,
            token_store,
            metrics_cache,
            exitnode_manager: Arc::new(ExitNodeManager::new(exitnode_cfg)),
            physnet_state: Arc::new(RwLock::new(saved.physnet)),
            bridge_state: Arc::new(RwLock::new(saved.bridge)),
            relay_remote: Arc::new(RwLock::new(saved.relay_remote)),
            log_collector,
            runtime_state_path,
        })
    }
}

// ── Runtime state persistence helper ─────────────────────────────────────────

impl AppState {
    /// Snapshot current bridge/physnet/relay state and persist it to disk.
    /// Call after every mutation so the state survives restarts.
    pub async fn persist_runtime_state(&self) {
        let snap = crate::runtime_state::RuntimeState {
            physnet: self.physnet_state.read().await.clone(),
            bridge: self.bridge_state.read().await.clone(),
            relay_remote: self.relay_remote.read().await.clone(),
        };
        crate::runtime_state::save(&self.runtime_state_path, &snap);
    }
}
