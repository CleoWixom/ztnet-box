use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub server: ServerConfig,
    pub zerotier: ZeroTierConfig,
    pub metrics: MetricsConfig,
    pub exitnode: ExitNodeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ZeroTierConfig {
    pub local: LocalConfig,
    pub central: CentralConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LocalConfig {
    pub api_url: String,
    pub token_file: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CentralConfig {
    pub base_url: String,
    pub tokens: Vec<CentralToken>,
    pub active_token_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentralToken {
    pub id: String,
    pub name: String,
    pub token: String,
    pub rate_limit: RateLimit,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RateLimit {
    /// 20 req/s
    Free,
    /// 100 req/s
    Paid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub prometheus_url: String,
    pub poll_interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ExitNodeConfig {
    pub nftables_preferred: bool,
}

// ── Defaults ──────────────────────────────────────────────────────────────────

#[allow(clippy::derivable_impls)]
impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            zerotier: ZeroTierConfig::default(),
            metrics: MetricsConfig::default(),
            exitnode: ExitNodeConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 3000,
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for ZeroTierConfig {
    fn default() -> Self {
        Self {
            local: LocalConfig::default(),
            central: CentralConfig::default(),
        }
    }
}

impl Default for LocalConfig {
    fn default() -> Self {
        Self {
            api_url: "http://127.0.0.1:9993".into(),
            token_file: PathBuf::from("/var/lib/zerotier-one/authtoken.secret"),
        }
    }
}

impl Default for CentralConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.zerotier.com/api/v1".into(),
            tokens: vec![],
            active_token_id: String::new(),
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            prometheus_url: "http://127.0.0.1:9993/metrics".into(),
            poll_interval_seconds: 5,
        }
    }
}

impl Default for ExitNodeConfig {
    fn default() -> Self {
        Self {
            nftables_preferred: true,
        }
    }
}

impl CentralToken {
    pub fn new(name: String, token: String, rate_limit: RateLimit) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            token,
            rate_limit,
            created_at: Utc::now(),
        }
    }

    /// Маскированный токен: первые 4 символа + ***
    pub fn masked_token(&self) -> String {
        let prefix = &self.token[..self.token.len().min(4)];
        format!("{prefix}***")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_serde_round_trip() {
        let cfg = Config::default();
        let yaml = serde_yaml::to_string(&cfg).unwrap();
        let back: Config = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(back.server.port, cfg.server.port);
        assert_eq!(back.server.host, cfg.server.host);
        assert_eq!(back.zerotier.local.api_url, cfg.zerotier.local.api_url);
    }

    #[test]
    fn central_token_masked() {
        let t = CentralToken::new("test".into(), "abcdefghijklmnop".into(), RateLimit::Free);
        let masked = t.masked_token();
        assert!(masked.starts_with("abcd"));
        assert!(masked.contains("***"));
        assert!(!masked.contains("efghijklmnop"));
    }

    #[test]
    fn rate_limit_serde() {
        let free = serde_json::to_string(&RateLimit::Free).unwrap();
        let paid = serde_json::to_string(&RateLimit::Paid).unwrap();
        assert_eq!(free, "\"free\"");
        assert_eq!(paid, "\"paid\"");
        let back: RateLimit = serde_json::from_str(&paid).unwrap();
        assert!(matches!(back, RateLimit::Paid));
    }
}
