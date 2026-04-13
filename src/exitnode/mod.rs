pub mod deps;
pub mod interfaces;
pub mod ndp;
pub mod platform;
pub mod rules;

use chrono::{DateTime, Utc};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{config::schema::ExitNodeConfig, server::error::ApiError};

use rules::{ExitNodeRules, FirewallBackend};

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Default)]
pub struct ExitNodeState {
    pub enabled: bool,
    /// ZeroTier interface name (e.g. "ztabcd1234e") — not the 16-char network ID.
    pub zt_interface: Option<String>,
    pub wan_interface: Option<String>,
    pub backend: Option<FirewallBackend>,
    pub enable_ipv6: bool,
    pub ipv6_prefix: Option<String>,
    pub applied_at: Option<DateTime<Utc>>,
}

// ── Manager ───────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct ExitNodeManager {
    state: Arc<RwLock<ExitNodeState>>,
    config: ExitNodeConfig,
}

impl ExitNodeManager {
    pub fn new(config: ExitNodeConfig) -> Self {
        Self {
            state: Arc::new(RwLock::new(ExitNodeState::default())),
            config,
        }
    }

    pub async fn status(&self) -> ExitNodeState {
        self.state.read().await.clone()
    }

    pub async fn enable(
        &self,
        zt_iface: String,
        wan_iface: String,
        enable_ipv6: bool,
        ipv6_prefix: Option<String>,
    ) -> Result<ExitNodeState, ApiError> {
        // Root check
        #[cfg(unix)]
        if !nix::unistd::getuid().is_root() {
            return Err(ApiError::ExitNode(
                "Root privileges required to configure firewall rules".into(),
            ));
        }

        // Platform check
        let plat = platform::check();
        if !plat.supported {
            return Err(ApiError::ExitNode(
                plat.reason.unwrap_or_else(|| "Unsupported platform".into()),
            ));
        }

        // Select backend
        let backend = select_backend(self.config.nftables_preferred);

        let rules = ExitNodeRules::new(zt_iface.clone(), wan_iface.clone(), backend)
            .with_ipv6(enable_ipv6, ipv6_prefix.clone());
        rules
            .apply()
            .map_err(|e| ApiError::ExitNode(e.to_string()))?;

        tracing::info!(
            zt = %zt_iface,
            wan = %wan_iface,
            ?backend,
            enable_ipv6,
            "exit node enabled"
        );

        let new_state = ExitNodeState {
            enabled: true,
            zt_interface: Some(zt_iface),
            wan_interface: Some(wan_iface),
            backend: Some(backend),
            enable_ipv6,
            ipv6_prefix,
            applied_at: Some(Utc::now()),
        };
        *self.state.write().await = new_state.clone();
        Ok(new_state)
    }

    pub async fn disable(&self) -> Result<(), ApiError> {
        #[cfg(unix)]
        if !nix::unistd::getuid().is_root() {
            return Err(ApiError::ExitNode(
                "Root privileges required to remove firewall rules".into(),
            ));
        }

        let st = self.state.read().await.clone();
        if !st.enabled {
            return Ok(()); // Already disabled
        }

        if let (Some(zt), Some(wan), Some(backend)) =
            (st.zt_interface, st.wan_interface, st.backend)
        {
            let rules =
                ExitNodeRules::new(zt, wan, backend).with_ipv6(st.enable_ipv6, st.ipv6_prefix);
            rules
                .remove()
                .map_err(|e| ApiError::ExitNode(e.to_string()))?;
        }

        tracing::info!("exit node disabled");
        *self.state.write().await = ExitNodeState::default();
        Ok(())
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn select_backend(prefer_nftables: bool) -> FirewallBackend {
    if prefer_nftables && which::which("nft").is_ok() {
        FirewallBackend::Nftables
    } else if which::which("iptables").is_ok() {
        FirewallBackend::Iptables
    } else if which::which("nft").is_ok() {
        FirewallBackend::Nftables
    } else {
        FirewallBackend::Iptables // будет ошибка при применении
    }
}
