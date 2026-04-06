use std::{collections::HashMap, sync::Arc, time::Instant};
use tokio::sync::RwLock;

pub struct MetricsCache {
    inner: Arc<RwLock<CacheInner>>,
}

struct CacheInner {
    data: HashMap<String, f64>,
    last_updated: Option<Instant>,
}

impl MetricsCache {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(CacheInner {
                data: HashMap::new(),
                last_updated: None,
            })),
        }
    }

    pub async fn update(&self, data: HashMap<String, f64>) {
        let mut inner = self.inner.write().await;
        inner.data = data;
        inner.last_updated = Some(Instant::now());
    }

    pub async fn get(&self) -> HashMap<String, f64> {
        self.inner.read().await.data.clone()
    }

    pub async fn age_secs(&self) -> Option<u64> {
        self.inner
            .read()
            .await
            .last_updated
            .map(|t| t.elapsed().as_secs())
    }
}

impl Default for MetricsCache {
    fn default() -> Self {
        Self::new()
    }
}
