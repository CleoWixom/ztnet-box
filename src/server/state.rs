use crate::{
    config::{schema::Config, ConfigError},
    metrics::cache::MetricsCache,
    zerotier::central::token_store::TokenStore,
};
use std::{path::PathBuf, sync::Arc};
use tokio::sync::RwLock;

#[derive(Debug, Default)]
pub struct ExitNodeState {
    pub enabled: bool,
    pub zt_iface: Option<String>,
    pub wan_iface: Option<String>,
}

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<RwLock<Config>>,
    pub config_path: PathBuf,
    pub token_store: Arc<TokenStore>,
    pub metrics_cache: Arc<MetricsCache>,
    pub exitnode: Arc<RwLock<ExitNodeState>>,
}

impl AppState {
    /// Стандартный конструктор — создаёт собственный MetricsCache.
    pub fn new(config: Config, config_path: PathBuf) -> Result<Self, ConfigError> {
        Self::new_with_cache(config, config_path, Arc::new(MetricsCache::new()))
    }

    /// Конструктор с внешним MetricsCache (для main.rs, где коллектор запущен отдельно).
    pub fn new_with_cache(
        config: Config,
        config_path: PathBuf,
        metrics_cache: Arc<MetricsCache>,
    ) -> Result<Self, ConfigError> {
        let tokens = config.zerotier.central.tokens.clone();
        let active_token_id = config.zerotier.central.active_token_id.clone();

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            config_path,
            token_store: Arc::new(TokenStore::new(tokens, active_token_id)),
            metrics_cache,
            exitnode: Arc::new(RwLock::new(ExitNodeState::default())),
        })
    }
}
