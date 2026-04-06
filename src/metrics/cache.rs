use chrono::{DateTime, Utc};
use serde::Serialize;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use super::parser::{self, MetricSample};

// ── Typed snapshot structs ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Default)]
pub struct PacketMetrics {
    pub rx_bytes: f64,
    pub tx_bytes: f64,
    pub rx_packets: f64,
    pub tx_packets: f64,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct LatencyMetrics {
    pub avg_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PeerMetric {
    pub node_id: String,
    pub status: String,
    pub latency_ms: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct NetworkMetric {
    pub network_id: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct ErrorMetrics {
    pub total: f64,
    pub by_type: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct MetricsSnapshot {
    pub packets: PacketMetrics,
    pub latency: LatencyMetrics,
    pub peers: Vec<PeerMetric>,
    pub networks: Vec<NetworkMetric>,
    pub errors: ErrorMetrics,
}

// ── Cache ─────────────────────────────────────────────────────────────────────

pub struct MetricsCache {
    parsed: Arc<RwLock<Option<MetricsSnapshot>>>,
    raw: Arc<RwLock<Option<String>>>,
    last_updated: Arc<RwLock<Option<DateTime<Utc>>>>,
    last_error: Arc<RwLock<Option<String>>>,
}

impl MetricsCache {
    pub fn new() -> Self {
        Self {
            parsed: Arc::new(RwLock::new(None)),
            raw: Arc::new(RwLock::new(None)),
            last_updated: Arc::new(RwLock::new(None)),
            last_error: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn update_from_raw(&self, text: String) {
        let samples = parser::parse(&text);
        let snapshot = build_snapshot(&samples);

        *self.raw.write().await = Some(text);
        *self.parsed.write().await = Some(snapshot);
        *self.last_updated.write().await = Some(Utc::now());
        *self.last_error.write().await = None;
    }

    pub async fn record_error(&self, err: String) {
        *self.last_error.write().await = Some(err);
    }

    pub async fn snapshot(&self) -> Option<MetricsSnapshot> {
        self.parsed.read().await.clone()
    }

    pub async fn raw_text(&self) -> Option<String> {
        self.raw.read().await.clone()
    }

    pub async fn last_updated(&self) -> Option<DateTime<Utc>> {
        *self.last_updated.read().await
    }

    pub async fn last_error(&self) -> Option<String> {
        self.last_error.read().await.clone()
    }
}

impl Default for MetricsCache {
    fn default() -> Self {
        Self::new()
    }
}

// ── Snapshot builder ──────────────────────────────────────────────────────────

fn build_snapshot(samples: &[MetricSample]) -> MetricsSnapshot {
    let mut snap = MetricsSnapshot::default();

    // Index samples by name for O(1) lookup
    let by_name: HashMap<&str, &MetricSample> =
        samples.iter().map(|s| (s.name.as_str(), s)).collect();

    // Packets
    snap.packets.rx_bytes = by_name.get("zt_packet_rx_bytes").map_or(0.0, |s| s.value);
    snap.packets.tx_bytes = by_name.get("zt_packet_tx_bytes").map_or(0.0, |s| s.value);
    snap.packets.rx_packets = by_name.get("zt_packet_rx").map_or(0.0, |s| s.value);
    snap.packets.tx_packets = by_name.get("zt_packet_tx").map_or(0.0, |s| s.value);

    // Latency
    snap.latency.avg_ms = by_name.get("zt_latency_avg").map_or(0.0, |s| s.value);
    snap.latency.min_ms = by_name.get("zt_latency_min").map_or(0.0, |s| s.value);
    snap.latency.max_ms = by_name.get("zt_latency_max").map_or(0.0, |s| s.value);

    // Peers — one sample per node
    let peers: Vec<PeerMetric> = samples
        .iter()
        .filter(|s| s.name == "zt_peer_latency")
        .map(|s| PeerMetric {
            node_id: s.labels.get("node_id").cloned().unwrap_or_default(),
            status: s.labels.get("status").cloned().unwrap_or_default(),
            latency_ms: s.value,
        })
        .collect();
    snap.peers = peers;

    // Networks — one sample per network
    let networks: Vec<NetworkMetric> = samples
        .iter()
        .filter(|s| s.name == "zt_network_status")
        .map(|s| NetworkMetric {
            network_id: s.labels.get("network_id").cloned().unwrap_or_default(),
            status: s.labels.get("status").cloned().unwrap_or_default(),
        })
        .collect();
    snap.networks = networks;

    // Errors
    let total: f64 = samples
        .iter()
        .filter(|s| s.name == "zt_packet_error")
        .map(|s| s.value)
        .sum();
    let by_type: HashMap<String, f64> = samples
        .iter()
        .filter(|s| s.name == "zt_packet_error")
        .filter_map(|s| s.labels.get("type").map(|t| (t.clone(), s.value)))
        .collect();
    snap.errors = ErrorMetrics { total, by_type };

    snap
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn cache_update_and_read() {
        let cache = MetricsCache::new();
        assert!(cache.snapshot().await.is_none());

        let raw = "zt_packet_rx_bytes 100\nzt_packet_tx_bytes 200\n";
        cache.update_from_raw(raw.to_string()).await;

        let snap = cache.snapshot().await.unwrap();
        assert_eq!(snap.packets.rx_bytes, 100.0);
        assert_eq!(snap.packets.tx_bytes, 200.0);
        assert!(cache.last_updated().await.is_some());
        assert!(cache.last_error().await.is_none());
    }

    #[tokio::test]
    async fn cache_peers_from_labels() {
        let cache = MetricsCache::new();
        let raw = r#"zt_peer_latency{node_id="aabbccddee",status="online"} 12.5"#;
        cache.update_from_raw(raw.to_string()).await;
        let snap = cache.snapshot().await.unwrap();
        assert_eq!(snap.peers.len(), 1);
        assert_eq!(snap.peers[0].node_id, "aabbccddee");
        assert_eq!(snap.peers[0].latency_ms, 12.5);
    }

    #[tokio::test]
    async fn cache_records_error() {
        let cache = MetricsCache::new();
        cache.record_error("connection refused".to_string()).await;
        assert_eq!(
            cache.last_error().await.as_deref(),
            Some("connection refused")
        );
    }
}
