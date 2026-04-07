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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::schema::Config;
    use std::sync::Mutex;

    // Serialize env-var tests to prevent races
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn env_override_host_and_port() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("ZT_SERVER_HOST", "0.0.0.0");
        std::env::set_var("ZT_SERVER_PORT", "8080");
        let mut cfg = Config::default();
        apply_env_overrides(&mut cfg);
        assert_eq!(cfg.server.host, "0.0.0.0");
        assert_eq!(cfg.server.port, 8080);
        std::env::remove_var("ZT_SERVER_HOST");
        std::env::remove_var("ZT_SERVER_PORT");
    }

    #[test]
    fn env_override_local_api_url() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("ZT_LOCAL_API_URL", "http://10.0.0.1:9993");
        let mut cfg = Config::default();
        apply_env_overrides(&mut cfg);
        assert_eq!(cfg.zerotier.local.api_url, "http://10.0.0.1:9993");
        std::env::remove_var("ZT_LOCAL_API_URL");
    }

    #[test]
    fn env_invalid_port_leaves_default() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("ZT_SERVER_PORT", "not_a_number");
        let mut cfg = Config::default();
        let orig = cfg.server.port;
        apply_env_overrides(&mut cfg);
        assert_eq!(cfg.server.port, orig);
        std::env::remove_var("ZT_SERVER_PORT");
    }

    #[test]
    fn no_env_vars_leaves_defaults() {
        let _guard = ENV_LOCK.lock().unwrap();
        // Ensure test env vars are absent
        for key in &[
            "ZT_SERVER_HOST",
            "ZT_SERVER_PORT",
            "ZT_LOCAL_API_URL",
            "ZT_LOCAL_TOKEN_FILE",
            "ZT_CENTRAL_BASE_URL",
        ] {
            std::env::remove_var(key);
        }
        let mut cfg = Config::default();
        let orig_host = cfg.server.host.clone();
        let orig_port = cfg.server.port;
        apply_env_overrides(&mut cfg);
        assert_eq!(cfg.server.host, orig_host);
        assert_eq!(cfg.server.port, orig_port);
    }
}
