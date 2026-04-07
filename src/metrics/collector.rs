use super::cache::MetricsCache;
use reqwest::Client;
use std::{sync::Arc, time::Duration};

/// Background metrics collector - use `MetricsCollector::start()` to launch.
pub struct MetricsCollector;

impl MetricsCollector {
    /// Запускает фоновый сбор метрик. Возвращает JoinHandle.
    pub fn start(
        url: String,
        interval_secs: u64,
        cache: Arc<MetricsCache>,
    ) -> tokio::task::JoinHandle<()> {
        let client = Client::new();
        let interval = Duration::from_secs(interval_secs.max(1));

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;
                match client.get(&url).send().await {
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
