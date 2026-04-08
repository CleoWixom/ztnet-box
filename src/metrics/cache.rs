//! Typed metrics cache.
//!
//! Maps raw Prometheus text from `http://localhost:9993/metrics` to typed structs.
//!
//! Official metric names (ZeroTier 1.14+):
//!
//! | Name                                 | Type      | Labels                         |
//! |--------------------------------------|-----------|--------------------------------|
//! | zt_packet                            | Counter   | packet_type, direction         |
//! | zt_packet_error                      | Counter   | error_type, direction          |
//! | zt_data                              | Counter   | protocol, direction            |
//! | zt_num_networks                      | Gauge     | —                              |
//! | zt_network_multicast_groups_subscribed | Gauge   | network_id                     |
//! | zt_network_packets                   | Counter   | network_id, direction          |
//! | zt_peer_latency                      | Histogram | node_id                        |
//! | zt_peer_path_count                   | Gauge     | node_id, status                |
//! | zt_peer_packets                      | Counter   | node_id, direction             |
//! | zt_peer_packet_errors                | Counter   | node_id                        |

use chrono::{DateTime, Utc};
use serde::Serialize;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use super::parser::{self, MetricSample};

// ── Typed snapshot structs ────────────────────────────────────────────────────

/// Aggregate traffic counters sourced from `zt_data` and `zt_packet`.
#[derive(Debug, Clone, Serialize, Default)]
pub struct PacketMetrics {
    /// Bytes received — sum of `zt_data{direction="rx"}` across protocols
    pub rx_bytes: f64,
    /// Bytes transmitted — sum of `zt_data{direction="tx"}` across protocols
    pub tx_bytes: f64,
    /// Packets received — sum of `zt_packet{direction="rx"}` across packet_types
    pub rx_packets: f64,
    /// Packets transmitted — sum of `zt_packet{direction="tx"}` across packet_types
    pub tx_packets: f64,
}

/// Aggregate latency computed from per-peer `zt_peer_latency` histogram sums/counts.
#[derive(Debug, Clone, Serialize, Default)]
pub struct LatencyMetrics {
    /// Mean latency across all peers (ms). 0 when no peers.
    pub avg_ms: f64,
    /// Number of peers included in the latency average.
    pub peer_count: u32,
}

/// Per-peer metrics derived from `zt_peer_latency`, `zt_peer_path_count`, `zt_peer_packets`.
#[derive(Debug, Clone, Serialize)]
pub struct PeerMetric {
    pub node_id: String,
    /// Average latency in ms (zt_peer_latency_sum / zt_peer_latency_count). None if no samples.
    pub latency_ms: Option<f64>,
    /// Active paths — from `zt_peer_path_count{node_id, status="active"}`
    pub active_paths: u32,
    /// Total paths (all statuses) — sum of `zt_peer_path_count{node_id}`
    pub total_paths: u32,
    /// Packets received from peer — `zt_peer_packets{node_id, direction="rx"}`
    pub rx_packets: f64,
    /// Packets sent to peer — `zt_peer_packets{node_id, direction="tx"}`
    pub tx_packets: f64,
    /// Packet errors — `zt_peer_packet_errors{node_id}`
    pub packet_errors: f64,
}

/// Per-network metrics from `zt_network_packets` and `zt_network_multicast_groups_subscribed`.
#[derive(Debug, Clone, Serialize)]
pub struct NetworkMetric {
    pub network_id: String,
    pub rx_packets: f64,
    pub tx_packets: f64,
    pub multicast_subscriptions: f64,
}

/// Aggregate error counters from `zt_packet_error{error_type, direction}`.
#[derive(Debug, Clone, Serialize, Default)]
pub struct ErrorMetrics {
    pub total: f64,
    /// Breakdown by `error_type` label
    pub by_type: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct MetricsSnapshot {
    pub packets: PacketMetrics,
    pub latency: LatencyMetrics,
    pub peers: Vec<PeerMetric>,
    pub networks: Vec<NetworkMetric>,
    pub errors: ErrorMetrics,
    /// Raw value of `zt_num_networks`
    pub num_networks: u32,
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

    // ── zt_data{protocol, direction} → bytes ─────────────────────────────────
    for s in samples.iter().filter(|s| s.name == "zt_data") {
        match s.labels.get("direction").map(|d| d.as_str()) {
            Some("rx") => snap.packets.rx_bytes += s.value,
            Some("tx") => snap.packets.tx_bytes += s.value,
            _ => {}
        }
    }

    // ── zt_packet{packet_type, direction} → packet counts ────────────────────
    for s in samples.iter().filter(|s| s.name == "zt_packet") {
        match s.labels.get("direction").map(|d| d.as_str()) {
            Some("rx") => snap.packets.rx_packets += s.value,
            Some("tx") => snap.packets.tx_packets += s.value,
            _ => {}
        }
    }

    // ── zt_num_networks ───────────────────────────────────────────────────────
    if let Some(s) = samples.iter().find(|s| s.name == "zt_num_networks") {
        snap.num_networks = s.value as u32;
    }

    // ── zt_packet_error{error_type, direction} ────────────────────────────────
    for s in samples.iter().filter(|s| s.name == "zt_packet_error") {
        snap.errors.total += s.value;
        if let Some(et) = s.labels.get("error_type") {
            *snap.errors.by_type.entry(et.clone()).or_default() += s.value;
        }
    }

    // ── Peers ─────────────────────────────────────────────────────────────────
    // Collect all known node_ids across all peer metrics
    let mut peer_node_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    for s in samples.iter() {
        if matches!(
            s.name.as_str(),
            "zt_peer_latency_sum"
                | "zt_peer_latency_count"
                | "zt_peer_path_count"
                | "zt_peer_packets"
                | "zt_peer_packet_errors"
        ) {
            if let Some(id) = s.labels.get("node_id") {
                peer_node_ids.insert(id.clone());
            }
        }
    }

    for node_id in &peer_node_ids {
        // Latency: avg = sum / count
        let lat_sum = samples
            .iter()
            .find(|s| s.name == "zt_peer_latency_sum" && s.labels.get("node_id") == Some(node_id))
            .map(|s| s.value);
        let lat_count = samples
            .iter()
            .find(|s| s.name == "zt_peer_latency_count" && s.labels.get("node_id") == Some(node_id))
            .map(|s| s.value);
        let latency_ms = match (lat_sum, lat_count) {
            (Some(sum), Some(cnt)) if cnt > 0.0 => Some(sum / cnt),
            _ => None,
        };

        // Path counts
        let active_paths = samples
            .iter()
            .filter(|s| {
                s.name == "zt_peer_path_count"
                    && s.labels.get("node_id") == Some(node_id)
                    && s.labels.get("status").map(|v| v.as_str()) == Some("active")
            })
            .map(|s| s.value as u32)
            .sum();
        let total_paths = samples
            .iter()
            .filter(|s| s.name == "zt_peer_path_count" && s.labels.get("node_id") == Some(node_id))
            .map(|s| s.value as u32)
            .sum();

        // Per-peer packets
        let rx_packets = samples
            .iter()
            .filter(|s| {
                s.name == "zt_peer_packets"
                    && s.labels.get("node_id") == Some(node_id)
                    && s.labels.get("direction").map(|v| v.as_str()) == Some("rx")
            })
            .map(|s| s.value)
            .sum();
        let tx_packets = samples
            .iter()
            .filter(|s| {
                s.name == "zt_peer_packets"
                    && s.labels.get("node_id") == Some(node_id)
                    && s.labels.get("direction").map(|v| v.as_str()) == Some("tx")
            })
            .map(|s| s.value)
            .sum();
        let packet_errors = samples
            .iter()
            .filter(|s| {
                s.name == "zt_peer_packet_errors" && s.labels.get("node_id") == Some(node_id)
            })
            .map(|s| s.value)
            .sum();

        snap.peers.push(PeerMetric {
            node_id: node_id.clone(),
            latency_ms,
            active_paths,
            total_paths,
            rx_packets,
            tx_packets,
            packet_errors,
        });
    }
    snap.peers.sort_by(|a, b| a.node_id.cmp(&b.node_id));

    // ── Aggregate latency across all peers ───────────────────────────────────
    let (total_sum, total_count): (f64, f64) = samples
        .iter()
        .filter(|s| s.name == "zt_peer_latency_sum" || s.name == "zt_peer_latency_count")
        .fold((0.0, 0.0), |(sum, cnt), s| {
            if s.name == "zt_peer_latency_sum" {
                (sum + s.value, cnt)
            } else {
                (sum, cnt + s.value)
            }
        });
    if total_count > 0.0 {
        snap.latency.avg_ms = total_sum / total_count;
    }
    snap.latency.peer_count = snap.peers.len() as u32;

    // ── Networks ──────────────────────────────────────────────────────────────
    // Collect unique network_ids from zt_network_packets
    let mut net_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    for s in samples.iter().filter(|s| {
        s.name == "zt_network_packets" || s.name == "zt_network_multicast_groups_subscribed"
    }) {
        if let Some(id) = s.labels.get("network_id") {
            net_ids.insert(id.clone());
        }
    }

    for network_id in &net_ids {
        let rx_packets = samples
            .iter()
            .filter(|s| {
                s.name == "zt_network_packets"
                    && s.labels.get("network_id") == Some(network_id)
                    && s.labels.get("direction").map(|v| v.as_str()) == Some("rx")
            })
            .map(|s| s.value)
            .sum();
        let tx_packets = samples
            .iter()
            .filter(|s| {
                s.name == "zt_network_packets"
                    && s.labels.get("network_id") == Some(network_id)
                    && s.labels.get("direction").map(|v| v.as_str()) == Some("tx")
            })
            .map(|s| s.value)
            .sum();
        let multicast_subscriptions = samples
            .iter()
            .find(|s| {
                s.name == "zt_network_multicast_groups_subscribed"
                    && s.labels.get("network_id") == Some(network_id)
            })
            .map_or(0.0, |s| s.value);

        snap.networks.push(NetworkMetric {
            network_id: network_id.clone(),
            rx_packets,
            tx_packets,
            multicast_subscriptions,
        });
    }
    snap.networks
        .sort_by(|a, b| a.network_id.cmp(&b.network_id));

    snap
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_METRICS: &str = r#"
# HELP zt_packet Total packets by type and direction
# TYPE zt_packet counter
zt_packet{packet_type="FRAME",direction="rx"} 1000
zt_packet{packet_type="FRAME",direction="tx"} 800
zt_packet{packet_type="NOP",direction="rx"} 50
# HELP zt_data Bytes by protocol and direction
# TYPE zt_data counter
zt_data{protocol="UDP",direction="rx"} 102400
zt_data{protocol="TCP",direction="rx"} 204800
zt_data{protocol="UDP",direction="tx"} 51200
# HELP zt_packet_error Errors by type and direction
# TYPE zt_packet_error counter
zt_packet_error{error_type="INVALID_REQUEST",direction="rx"} 5
zt_packet_error{error_type="TIMEOUT",direction="rx"} 2
# HELP zt_num_networks Number of joined networks
# TYPE zt_num_networks gauge
zt_num_networks 2
# HELP zt_network_packets Packets per network
# TYPE zt_network_packets counter
zt_network_packets{network_id="8056c2e21c000001",direction="rx"} 500
zt_network_packets{network_id="8056c2e21c000001",direction="tx"} 400
zt_network_multicast_groups_subscribed{network_id="8056c2e21c000001"} 3
# HELP zt_peer_latency Peer latency histogram
# TYPE zt_peer_latency histogram
zt_peer_latency_sum{node_id="aabbccddee"} 125.0
zt_peer_latency_count{node_id="aabbccddee"} 5
zt_peer_latency_sum{node_id="1122334455"} 60.0
zt_peer_latency_count{node_id="1122334455"} 3
# HELP zt_peer_path_count Paths per peer
# TYPE zt_peer_path_count gauge
zt_peer_path_count{node_id="aabbccddee",status="active"} 2
zt_peer_path_count{node_id="aabbccddee",status="inactive"} 1
zt_peer_path_count{node_id="1122334455",status="active"} 1
# HELP zt_peer_packets Packets per peer
# TYPE zt_peer_packets counter
zt_peer_packets{node_id="aabbccddee",direction="rx"} 300
zt_peer_packets{node_id="aabbccddee",direction="tx"} 250
# HELP zt_peer_packet_errors Errors per peer
# TYPE zt_peer_packet_errors counter
zt_peer_packet_errors{node_id="aabbccddee"} 1
"#;

    #[tokio::test]
    async fn bytes_from_zt_data() {
        let cache = MetricsCache::new();
        cache.update_from_raw(SAMPLE_METRICS.to_string()).await;
        let snap = cache.snapshot().await.unwrap();
        // rx_bytes = UDP(102400) + TCP(204800)
        assert_eq!(snap.packets.rx_bytes, 307200.0);
        // tx_bytes = UDP(51200)
        assert_eq!(snap.packets.tx_bytes, 51200.0);
    }

    #[tokio::test]
    async fn packets_from_zt_packet() {
        let cache = MetricsCache::new();
        cache.update_from_raw(SAMPLE_METRICS.to_string()).await;
        let snap = cache.snapshot().await.unwrap();
        // rx = FRAME(1000) + NOP(50)
        assert_eq!(snap.packets.rx_packets, 1050.0);
        assert_eq!(snap.packets.tx_packets, 800.0);
    }

    #[tokio::test]
    async fn num_networks_gauge() {
        let cache = MetricsCache::new();
        cache.update_from_raw(SAMPLE_METRICS.to_string()).await;
        let snap = cache.snapshot().await.unwrap();
        assert_eq!(snap.num_networks, 2);
    }

    #[tokio::test]
    async fn per_peer_latency_from_histogram() {
        let cache = MetricsCache::new();
        cache.update_from_raw(SAMPLE_METRICS.to_string()).await;
        let snap = cache.snapshot().await.unwrap();
        assert_eq!(snap.peers.len(), 2);
        let peer = snap
            .peers
            .iter()
            .find(|p| p.node_id == "aabbccddee")
            .unwrap();
        // avg = 125.0 / 5 = 25.0 ms
        assert_eq!(peer.latency_ms, Some(25.0));
    }

    #[tokio::test]
    async fn per_peer_path_counts() {
        let cache = MetricsCache::new();
        cache.update_from_raw(SAMPLE_METRICS.to_string()).await;
        let snap = cache.snapshot().await.unwrap();
        let peer = snap
            .peers
            .iter()
            .find(|p| p.node_id == "aabbccddee")
            .unwrap();
        assert_eq!(peer.active_paths, 2);
        assert_eq!(peer.total_paths, 3); // active(2) + inactive(1)
    }

    #[tokio::test]
    async fn per_peer_packets() {
        let cache = MetricsCache::new();
        cache.update_from_raw(SAMPLE_METRICS.to_string()).await;
        let snap = cache.snapshot().await.unwrap();
        let peer = snap
            .peers
            .iter()
            .find(|p| p.node_id == "aabbccddee")
            .unwrap();
        assert_eq!(peer.rx_packets, 300.0);
        assert_eq!(peer.tx_packets, 250.0);
        assert_eq!(peer.packet_errors, 1.0);
    }

    #[tokio::test]
    async fn aggregate_latency_across_peers() {
        let cache = MetricsCache::new();
        cache.update_from_raw(SAMPLE_METRICS.to_string()).await;
        let snap = cache.snapshot().await.unwrap();
        // total_sum = 125 + 60 = 185, total_count = 5 + 3 = 8 → avg = 185/8 = 23.125
        assert!((snap.latency.avg_ms - 23.125).abs() < 0.001);
        assert_eq!(snap.latency.peer_count, 2);
    }

    #[tokio::test]
    async fn network_packets_from_zt_network_packets() {
        let cache = MetricsCache::new();
        cache.update_from_raw(SAMPLE_METRICS.to_string()).await;
        let snap = cache.snapshot().await.unwrap();
        assert_eq!(snap.networks.len(), 1);
        let net = &snap.networks[0];
        assert_eq!(net.network_id, "8056c2e21c000001");
        assert_eq!(net.rx_packets, 500.0);
        assert_eq!(net.multicast_subscriptions, 3.0);
    }

    #[tokio::test]
    async fn errors_use_error_type_label() {
        let cache = MetricsCache::new();
        cache.update_from_raw(SAMPLE_METRICS.to_string()).await;
        let snap = cache.snapshot().await.unwrap();
        assert_eq!(snap.errors.total, 7.0);
        assert_eq!(*snap.errors.by_type.get("INVALID_REQUEST").unwrap(), 5.0);
        assert_eq!(*snap.errors.by_type.get("TIMEOUT").unwrap(), 2.0);
    }

    #[tokio::test]
    async fn empty_metrics_returns_defaults() {
        let cache = MetricsCache::new();
        cache.update_from_raw("# empty\n".to_string()).await;
        let snap = cache.snapshot().await.unwrap();
        assert_eq!(snap.packets.rx_bytes, 0.0);
        assert!(snap.peers.is_empty());
        assert!(snap.networks.is_empty());
        assert_eq!(snap.errors.total, 0.0);
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
