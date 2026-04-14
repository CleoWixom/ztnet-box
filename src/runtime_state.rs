//! Persistent runtime state — bridge / physnet / relay.
//!
//! Saves to and restores from a JSON file so that ztnet-box survives restarts
//! without losing track of what it configured in the OS.
//!
//! **File location** (first writable path wins):
//!   1. `$ZTNET_STATE_FILE` env var
//!   2. `/var/lib/ztnet-box/state.json`   (system install)
//!   3. `~/.local/share/ztnet-box/state.json`  (user install / dev)
//!
//! The file is written atomically (write-then-rename) to prevent corruption on
//! power loss or SIGKILL.

use crate::{bridge::BridgeState, physnet::PhysNetState, relay::RemoteRelayInfo};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ── Schema ────────────────────────────────────────────────────────────────────

/// Everything that needs to survive a restart.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuntimeState {
    pub physnet: PhysNetState,
    pub bridge: BridgeState,
    pub relay_remote: Option<RemoteRelayInfo>,
}

// ── Path resolution ───────────────────────────────────────────────────────────

pub fn state_path() -> PathBuf {
    // 1. Explicit override (tests / custom installs)
    if let Ok(p) = std::env::var("ZTNET_STATE_FILE") {
        return PathBuf::from(p);
    }
    // 2. System path (requires root)
    let system = PathBuf::from("/var/lib/ztnet-box/state.json");
    if system.parent().map(|d| d.exists()).unwrap_or(false) {
        return system;
    }
    // 3. XDG user data dir
    let xdg = std::env::var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs_home()
                .map(|h| h.join(".local").join("share"))
                .unwrap_or_else(|| PathBuf::from("."))
        });
    xdg.join("ztnet-box").join("state.json")
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from).or_else(|| {
        // Fallback: parse /etc/passwd for current UID
        None
    })
}

// ── Load ─────────────────────────────────────────────────────────────────────

/// Load persisted state. Returns `Default` if the file doesn't exist yet.
pub fn load(path: &Path) -> RuntimeState {
    match std::fs::read_to_string(path) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_else(|e| {
            tracing::warn!(path = %path.display(), error = %e, "state.json parse error — using defaults");
            RuntimeState::default()
        }),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => RuntimeState::default(),
        Err(e) => {
            tracing::warn!(path = %path.display(), error = %e, "cannot read state.json — using defaults");
            RuntimeState::default()
        }
    }
}

// ── Save ─────────────────────────────────────────────────────────────────────

/// Persist current runtime state atomically (write to `.tmp`, then rename).
pub fn save(path: &Path, state: &RuntimeState) {
    if let Err(e) = save_inner(path, state) {
        tracing::warn!(path = %path.display(), error = %e, "failed to persist runtime state");
    }
}

fn save_inner(path: &Path, state: &RuntimeState) -> std::io::Result<()> {
    // Ensure parent directory exists
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let json = serde_json::to_string_pretty(state)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    // Write to temp file, then rename — atomic on POSIX
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, &json)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let dir = std::env::temp_dir().join(format!(
            "ztnet-state-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .subsec_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("state.json");

        let mut st = RuntimeState::default();
        st.physnet.enabled = true;
        save(&path, &st);

        let loaded = load(&path);
        assert!(loaded.physnet.enabled);
        assert!(!loaded.bridge.enabled);
        assert!(loaded.relay_remote.is_none());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn missing_file_returns_default() {
        let path = PathBuf::from("/tmp/ztnet-no-such-file-state.json");
        let st = load(&path);
        assert!(!st.physnet.enabled);
    }
}
