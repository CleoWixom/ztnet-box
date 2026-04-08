use super::cache::MetricsCache;
use reqwest::Client;
use std::{path::PathBuf, sync::Arc, time::Duration};

/// Background metrics collector. Launch with `MetricsCollector::start()`.
pub struct MetricsCollector;

impl MetricsCollector {
    /// Spawn a background task that polls the ZeroTier metrics endpoint.
    ///
    /// `metricstoken_file` — path to `metricstoken.secret` (Bearer auth).
    /// If the file doesn't exist or is empty, auth header is omitted.
    pub fn start(
        url: String,
        interval_secs: u64,
        cache: Arc<MetricsCache>,
        metricstoken_file: PathBuf,
    ) -> tokio::task::JoinHandle<()> {
        let client = Client::new();
        let interval = Duration::from_secs(interval_secs.max(1));

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;

                // Read token from file each tick so hot-reload works without restart
                let token = read_token(&metricstoken_file);

                let req = client.get(&url);
                let req = match &token {
                    Some(t) => req.bearer_auth(t),
                    None => req,
                };

                match req.send().await {
                    Ok(resp) => match resp.text().await {
                        Ok(text) => cache.update_from_raw(text).await,
                        Err(e) => {
                            tracing::warn!(error = %e, "metrics: failed to read response body");
                            cache.record_error(e.to_string()).await;
                        }
                    },
                    Err(e) => {
                        tracing::warn!(error = %e, url = %url, "metrics: fetch failed");
                        cache.record_error(e.to_string()).await;
                    }
                }
            }
        })
    }
}

/// Read and trim the metrics token from disk. Returns None if missing / empty.
fn read_token(path: &PathBuf) -> Option<String> {
    let token = std::fs::read_to_string(path).ok()?;
    let token = token.trim().to_string();
    if token.is_empty() {
        None
    } else {
        Some(token)
    }
}
