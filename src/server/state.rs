use crate::{
    bridge::BridgeState,
    config::{schema::Config, ConfigError},
    exitnode::ExitNodeManager,
    metrics::cache::MetricsCache,
    physnet::PhysNetState,
    relay::RemoteRelayInfo,
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

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            config_path,
            token_store,
            metrics_cache,
            exitnode_manager: Arc::new(ExitNodeManager::new(exitnode_cfg)),
            physnet_state: Arc::new(RwLock::new(PhysNetState::default())),
            bridge_state: Arc::new(RwLock::new(BridgeState::default())),
            relay_remote: Arc::new(RwLock::new(None)),
            log_collector,
        })
    }
}
