use super::schema::Config;
use std::env;

/// Применяет переменные окружения поверх загруженного конфига.
pub fn apply_env_overrides(cfg: &mut Config) {
    if let Ok(v) = env::var("ZT_SERVER_HOST") {
        cfg.server.host = v;
    }
    if let Ok(v) = env::var("ZT_SERVER_PORT") {
        if let Ok(port) = v.parse::<u16>() {
            cfg.server.port = port;
        }
    }
    if let Ok(v) = env::var("ZT_LOCAL_API_URL") {
        cfg.zerotier.local.api_url = v;
    }
    if let Ok(v) = env::var("ZT_LOCAL_TOKEN_FILE") {
        cfg.zerotier.local.token_file = v.into();
    }
    if let Ok(v) = env::var("ZT_CENTRAL_BASE_URL") {
        cfg.zerotier.central.base_url = v;
    }
}
