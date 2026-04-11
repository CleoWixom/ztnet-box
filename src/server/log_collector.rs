//! In-process log collector: buffers recent log entries and broadcasts them
//! to SSE clients via `GET /api/logs/stream`.
//!
//! Implemented as a custom [`tracing::Layer`] that intercepts every `tracing`
//! event, converts it into a [`LogEntry`], pushes it into a ring buffer, and
//! sends it over a `tokio::sync::broadcast` channel.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;
use tracing::Level;

// ── Types ─────────────────────────────────────────────────────────────────────

/// Severity level, mirrors `tracing::Level` but serialisable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error = 4,
    Warn = 3,
    Info = 2,
    Debug = 1,
    Trace = 0,
}

impl LogLevel {
    pub fn from_tracing(level: &Level) -> Self {
        match *level {
            Level::ERROR => LogLevel::Error,
            Level::WARN => LogLevel::Warn,
            Level::INFO => LogLevel::Info,
            Level::DEBUG => LogLevel::Debug,
            Level::TRACE => LogLevel::Trace,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            LogLevel::Error => "error",
            LogLevel::Warn => "warn",
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
            LogLevel::Trace => "trace",
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for LogLevel {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "error" => Ok(LogLevel::Error),
            "warn" | "warning" => Ok(LogLevel::Warn),
            "info" => Ok(LogLevel::Info),
            "debug" => Ok(LogLevel::Debug),
            "trace" => Ok(LogLevel::Trace),
            other => Err(format!("unknown log level '{other}'")),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub target: String,
    pub message: String,
}

// ── Collector ─────────────────────────────────────────────────────────────────

const BUFFER_CAP: usize = 500;
const BROADCAST_CAP: usize = 256;

/// Shared state: ring buffer + broadcast sender.
#[derive(Debug)]
struct Inner {
    buffer: Mutex<VecDeque<LogEntry>>,
    tx: broadcast::Sender<LogEntry>,
    min_level: Mutex<LogLevel>,
}

/// The log collector — cheap to clone (Arc inside).
#[derive(Clone, Debug)]
pub struct LogCollector {
    inner: Arc<Inner>,
}

impl LogCollector {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(BROADCAST_CAP);
        Self {
            inner: Arc::new(Inner {
                buffer: Mutex::new(VecDeque::with_capacity(BUFFER_CAP)),
                tx,
                min_level: Mutex::new(LogLevel::Info),
            }),
        }
    }

    /// Returns a copy of buffered entries, optionally filtered by minimum level.
    pub fn entries(&self, min_level: Option<LogLevel>, limit: usize) -> Vec<LogEntry> {
        let buf = self.inner.buffer.lock().expect("log buffer lock");
        let min = min_level.unwrap_or(LogLevel::Trace);
        buf.iter()
            .filter(|e| e.level >= min)
            .rev()
            .take(limit)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Subscribe to the live stream.
    pub fn subscribe(&self) -> broadcast::Receiver<LogEntry> {
        self.inner.tx.subscribe()
    }

    /// Clear the ring buffer.
    pub fn clear(&self) {
        self.inner
            .buffer
            .lock()
            .expect("log buffer lock")
            .clear();
    }

    /// Change the minimum capture level at runtime.
    pub fn set_level(&self, level: LogLevel) {
        *self.inner.min_level.lock().expect("min_level lock") = level;
    }

    /// Current minimum capture level.
    pub fn current_level(&self) -> LogLevel {
        *self.inner.min_level.lock().expect("min_level lock")
    }

    /// Push an entry (used by the tracing Layer).
    fn push(&self, entry: LogEntry) {
        // Drop if below current min level
        let min = *self.inner.min_level.lock().expect("min_level lock");
        if entry.level < min {
            return;
        }

        // Broadcast (ignore SendError — no subscribers is fine)
        let _ = self.inner.tx.send(entry.clone());

        // Ring buffer
        let mut buf = self.inner.buffer.lock().expect("log buffer lock");
        if buf.len() == BUFFER_CAP {
            buf.pop_front();
        }
        buf.push_back(entry);
    }
}

impl Default for LogCollector {
    fn default() -> Self {
        Self::new()
    }
}

// ── tracing Layer ─────────────────────────────────────────────────────────────

/// A [`tracing::Layer`] that feeds every event into the [`LogCollector`].
pub struct CollectorLayer {
    collector: LogCollector,
}

impl CollectorLayer {
    pub fn new(collector: LogCollector) -> Self {
        Self { collector }
    }
}

impl<S> tracing_subscriber::Layer<S> for CollectorLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let level = LogLevel::from_tracing(event.metadata().level());
        let target = event.metadata().target().to_string();

        // Extract the message field (the first `message` or `%message` field)
        let mut visitor = MessageVisitor(String::new());
        event.record(&mut visitor);

        self.collector.push(LogEntry {
            timestamp: Utc::now(),
            level,
            target,
            message: visitor.0,
        });
    }
}

/// Extracts the `message` field value from a tracing event.
struct MessageVisitor(String);

impl tracing::field::Visit for MessageVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.0 = format!("{value:?}");
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.0 = value.to_string();
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_retrieve() {
        let c = LogCollector::new();
        c.push(LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Info,
            target: "test".into(),
            message: "hello".into(),
        });
        let entries = c.entries(None, 100);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].message, "hello");
    }

    #[test]
    fn level_filter() {
        let c = LogCollector::new();
        c.set_level(LogLevel::Warn);
        c.push(LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Info,
            target: "t".into(),
            message: "filtered".into(),
        });
        c.push(LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Error,
            target: "t".into(),
            message: "kept".into(),
        });
        let entries = c.entries(None, 100);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].message, "kept");
    }

    #[test]
    fn clear_buffer() {
        let c = LogCollector::new();
        c.set_level(LogLevel::Trace);
        for i in 0..5 {
            c.push(LogEntry {
                timestamp: Utc::now(),
                level: LogLevel::Info,
                target: "t".into(),
                message: format!("msg {i}"),
            });
        }
        assert_eq!(c.entries(None, 100).len(), 5);
        c.clear();
        assert_eq!(c.entries(None, 100).len(), 0);
    }

    #[test]
    fn ring_buffer_evicts_oldest() {
        let c = LogCollector::new();
        c.set_level(LogLevel::Trace);
        for i in 0..BUFFER_CAP + 10 {
            c.push(LogEntry {
                timestamp: Utc::now(),
                level: LogLevel::Info,
                target: "t".into(),
                message: format!("msg {i}"),
            });
        }
        let entries = c.entries(None, BUFFER_CAP + 100);
        assert_eq!(entries.len(), BUFFER_CAP);
        // Oldest entries were evicted — first kept is msg 10
        assert_eq!(entries[0].message, "msg 10");
    }

    #[test]
    fn min_level_filter_on_entries() {
        let c = LogCollector::new();
        c.set_level(LogLevel::Trace);
        for &lvl in &[LogLevel::Debug, LogLevel::Info, LogLevel::Warn, LogLevel::Error] {
            c.push(LogEntry {
                timestamp: Utc::now(),
                level: lvl,
                target: "t".into(),
                message: format!("{lvl}"),
            });
        }
        let warn_up = c.entries(Some(LogLevel::Warn), 100);
        assert_eq!(warn_up.len(), 2);
        assert_eq!(warn_up[0].level, LogLevel::Warn);
        assert_eq!(warn_up[1].level, LogLevel::Error);
    }

    #[test]
    fn log_level_parse_roundtrip() {
        for s in &["error", "warn", "info", "debug", "trace"] {
            let lvl: LogLevel = s.parse().unwrap();
            assert_eq!(lvl.as_str(), *s);
        }
        assert!("bad".parse::<LogLevel>().is_err());
    }

    #[test]
    fn subscribe_receives_entry() {
        let c = LogCollector::new();
        c.set_level(LogLevel::Trace);
        let mut rx = c.subscribe();
        c.push(LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Info,
            target: "t".into(),
            message: "streamed".into(),
        });
        let got = rx.try_recv().expect("should receive entry");
        assert_eq!(got.message, "streamed");
    }
}
