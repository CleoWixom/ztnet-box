use super::{cache::MetricsCache, parser};
use reqwest::Client;
use std::{sync::Arc, time::Duration};

pub struct MetricsCollector {
    client: Client,
    url: String,
    interval: Duration,
    pub cache: Arc<MetricsCache>,
}

impl MetricsCollector {
    pub fn new(url: String, interval_secs: u64, cache: Arc<MetricsCache>) -> Self {
        Self {
            client: Client::new(),
            url,
            interval: Duration::from_secs(interval_secs),
            cache,
        }
    }

    /// Запустить фоновый сбор метрик.
    pub fn spawn(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                match self.client.get(&self.url).send().await {
                    Ok(resp) => {
                        if let Ok(text) = resp.text().await {
                            let data = parser::parse(&text);
                            self.cache.update(data).await;
                        }
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "Failed to fetch metrics");
                    }
                }
                tokio::time::sleep(self.interval).await;
            }
        })
    }
}
