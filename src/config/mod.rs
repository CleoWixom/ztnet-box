pub mod env;
pub mod schema;

pub use schema::Config;

use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("Validation error: {0}")]
    Validation(String),
}

pub type Result<T> = std::result::Result<T, ConfigError>;

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let mut cfg: Config = serde_yaml::from_str(&content)?;
            cfg.validate()?;
            env::apply_env_overrides(&mut cfg);
            Ok(cfg)
        } else {
            tracing::warn!(path = %path.display(), "Config file not found, using defaults");
            let mut cfg = Config::default();
            env::apply_env_overrides(&mut cfg);
            Ok(cfg)
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn find_config_file() -> PathBuf {
        let candidates = [
            PathBuf::from("config.yml"),
            dirs_config_path(),
            PathBuf::from("/etc/ztnet-box/config.yml"),
        ];
        for path in &candidates {
            if path.exists() {
                return path.clone();
            }
        }
        // Если не найден — вернуть первый (будет создан с defaults)
        PathBuf::from("config.yml")
    }

    fn validate(&self) -> Result<()> {
        if self.server.port == 0 {
            return Err(ConfigError::Validation(
                "server.port must be 1–65535".into(),
            ));
        }
        if self.server.host.is_empty() {
            return Err(ConfigError::Validation(
                "server.host must not be empty".into(),
            ));
        }
        Ok(())
    }
}

fn dirs_config_path() -> PathBuf {
    if let Some(home) = std::env::var_os("HOME") {
        PathBuf::from(home)
            .join(".config")
            .join("ztnet-box")
            .join("config.yml")
    } else {
        PathBuf::from("/etc/ztnet-box/config.yml")
    }
}
