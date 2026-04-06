use crate::{
    config::{schema::Config, ConfigError},
    metrics::cache::MetricsCache,
    zerotier::central::token_store::TokenStore,
};
use std::{path::PathBuf, sync::Arc};
use tokio::sync::RwLock;

/// Состояние Exit Node (активен/выключен + интерфейсы).
#[derive(Debug, Default)]
pub struct ExitNodeState {
    pub enabled: bool,
    pub zt_iface: Option<String>,
    pub wan_iface: Option<String>,
}

/// Глобальное состояние приложения, разделяемое между хэндлерами.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<RwLock<Config>>,
    pub config_path: PathBuf,
    pub token_store: Arc<TokenStore>,
    pub metrics_cache: Arc<MetricsCache>,
    pub exitnode: Arc<RwLock<ExitNodeState>>,
}

impl AppState {
    pub fn new(config: Config, config_path: PathBuf) -> Result<Self, ConfigError> {
        let tokens = config.zerotier.central.tokens.clone();
        let active_token_id = config.zerotier.central.active_token_id.clone();

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            config_path,
            token_store: Arc::new(TokenStore::new(tokens, active_token_id)),
            metrics_cache: Arc::new(MetricsCache::new()),
            exitnode: Arc::new(RwLock::new(ExitNodeState::default())),
        })
    }
}
