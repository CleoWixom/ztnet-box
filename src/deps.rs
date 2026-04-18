//! Startup dependency check — ensures `zerotier-one` is installed and running.
//!
//! Called once from `main()` before the HTTP server starts.
//!
//! Sequence:
//!  1. Detect `zerotier-one` binary via PATH.
//!  2. If absent → install via the system package manager (apt/dnf/pacman).
//!  3. After install → patch the systemd service file so the daemon starts
//!     with the `-U` flag (`ExecStart=/usr/sbin/zerotier-one -U`).
//!  4. Enable and start the `zerotier-one` service.
//!  5. Wait up to 15 s for the daemon socket to appear.
//!  6. If any step fails → return `Err` so `main()` can log and abort.
//!
//! The `-U` flag runs ZeroTier in "unprivileged" mode:
//!   - No `CAP_NET_ADMIN` required after the TUN/TAP device is opened.
//!   - Still needs root *once* to create the interface; subsequent restarts
//!     can run as any user. Most distro packages already handle this, but
//!     the flag is the canonical way to signal the intent.

use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
use tracing::{error, info, warn};
use which::which;

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum DepsError {
    #[error("No supported package manager found (apt-get / dnf / pacman)")]
    NoPackageManager,
    #[error("zerotier-one install failed: {0}")]
    InstallFailed(String),
    #[error("Service file patch failed: {0}")]
    ServicePatchFailed(String),
    #[error("Failed to enable/start zerotier-one.service: {0}")]
    ServiceStartFailed(String),
    #[error("zerotier-one daemon socket not ready after {0}s")]
    DaemonNotReady(u64),
    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),
}

// ── Public entry point ────────────────────────────────────────────────────────

/// Run at startup. Returns `Ok(())` when zerotier-one is installed and running.
/// Returns `Err(DepsError)` if something could not be fixed automatically.
///
/// Set `ZTNET_SKIP_DEPS=1` to skip all checks (useful in CI / manual-testing
/// environments where zerotier-one is managed externally by the workflow).
pub fn ensure() -> Result<(), DepsError> {
    if std::env::var("ZTNET_SKIP_DEPS").as_deref() == Ok("1") {
        info!("deps: ZTNET_SKIP_DEPS=1 — skipping zerotier-one dependency check");
        return Ok(());
    }

    info!("deps: checking zerotier-one…");

    if !is_supported_platform() {
        let os = std::env::consts::OS;
        warn!("deps: platform '{os}' — skipping zerotier-one startup check");
        return Ok(());
    }

    let already_installed = which("zerotier-one").is_ok();

    if !already_installed {
        info!("deps: zerotier-one not found — installing…");
        let pm = detect_pm().ok_or(DepsError::NoPackageManager)?;
        install(pm)?;
        // After install, patch the service file to add -U flag
        patch_service_file()?;
        info!("deps: zerotier-one installed and service file patched");
    } else {
        info!("deps: zerotier-one is already installed");
        // Idempotent patch: ensure -U is present even on existing installs
        if let Err(e) = patch_service_file() {
            // Non-fatal: warn but continue if patching fails on existing install
            warn!("deps: could not patch service file: {e} — continuing");
        }
    }

    // Reload systemd to pick up any service file changes
    systemctl_daemon_reload();

    // Enable + start
    enable_and_start()?;

    // Wait for the daemon socket / authtoken to appear
    wait_for_daemon(15)?;

    let version = read_version().unwrap_or_else(|| "unknown".into());
    info!("deps: zerotier-one ready (version {version})");
    Ok(())
}

// ── Platform guard ────────────────────────────────────────────────────────────

fn is_supported_platform() -> bool {
    cfg!(any(target_os = "linux", target_os = "macos"))
}

// ── Package manager detection ─────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
enum Pm {
    Apt,
    Dnf,
    Pacman,
}

fn detect_pm() -> Option<Pm> {
    if which("apt-get").is_ok() {
        return Some(Pm::Apt);
    }
    if which("dnf").is_ok() || which("yum").is_ok() {
        return Some(Pm::Dnf);
    }
    if which("pacman").is_ok() {
        return Some(Pm::Pacman);
    }
    None
}

// ── Installation ──────────────────────────────────────────────────────────────

fn install(pm: Pm) -> Result<(), DepsError> {
    let ok = match pm {
        Pm::Apt => {
            // Refresh package index first so the package can be found on fresh systems
            let _ = run_cmd("apt-get", &["update", "-qq"]);
            run_cmd("apt-get", &["install", "-y", "zerotier-one"])
        }
        Pm::Dnf => {
            if which("dnf").is_ok() {
                run_cmd("dnf", &["install", "-y", "zerotier-one"])
            } else {
                run_cmd("yum", &["install", "-y", "zerotier-one"])
            }
        }
        Pm::Pacman => run_cmd("pacman", &["-S", "--noconfirm", "zerotier-one"]),
    };

    if ok {
        info!("deps: zerotier-one installed successfully via {:?}", pm);
        Ok(())
    } else {
        let msg = format!(
            "Package manager {:?} exited non-zero. \
             Try manually: curl -s https://install.zerotier.com | sudo bash",
            pm
        );
        error!("deps: {msg}");
        Err(DepsError::InstallFailed(msg))
    }
}

// ── Service file patching ─────────────────────────────────────────────────────

/// Candidate paths for the zerotier-one systemd unit file.
/// Different distros install it in different locations.
const SERVICE_CANDIDATES: &[&str] = &[
    "/lib/systemd/system/zerotier-one.service",
    "/usr/lib/systemd/system/zerotier-one.service",
    "/etc/systemd/system/zerotier-one.service",
];

/// Ensure `ExecStart=…zerotier-one` has the `-U` flag appended.
///
/// Replaces:
///   `ExecStart=/usr/sbin/zerotier-one`
/// with:
///   `ExecStart=/usr/sbin/zerotier-one -U`
///
/// Idempotent: if `-U` is already present nothing is changed.
fn patch_service_file() -> Result<(), DepsError> {
    let path = find_service_file().ok_or_else(|| {
        DepsError::ServicePatchFailed(format!(
            "service file not found in any of {:?}",
            SERVICE_CANDIDATES
        ))
    })?;

    patch_exec_start(&path)
}

fn find_service_file() -> Option<PathBuf> {
    // First: check the drop-in override directory (highest priority)
    let override_dir = Path::new("/etc/systemd/system/zerotier-one.service.d");
    if override_dir.exists() {
        // If there's already a drop-in, we may be done — check for -U there
        let dropin = override_dir.join("exec.conf");
        if dropin.exists() {
            if let Ok(content) = std::fs::read_to_string(&dropin) {
                if content.contains("-U") {
                    return None; // Already patched via drop-in, nothing to do
                }
            }
        }
    }
    // Otherwise find the main service file
    SERVICE_CANDIDATES
        .iter()
        .map(Path::new)
        .find(|p| p.exists())
        .map(PathBuf::from)
}

fn patch_exec_start(path: &Path) -> Result<(), DepsError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| DepsError::ServicePatchFailed(format!("read {}: {e}", path.display())))?;

    // Check if already patched
    if content.contains("zerotier-one -U") {
        info!("deps: service file already has -U flag: {}", path.display());
        return Ok(());
    }

    // Patch: append -U to the ExecStart line
    // Handles both absolute paths and quoted variants:
    //   ExecStart=/usr/sbin/zerotier-one
    //   ExecStart=/usr/sbin/zerotier-one --other-flags
    let patched = content
        .lines()
        .map(|line| {
            let trimmed = line.trim_start();
            if trimmed.starts_with("ExecStart=")
                && trimmed.contains("zerotier-one")
                && !trimmed.contains("-U")
            {
                // Append -U if not already present
                format!("{line} -U")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Preserve trailing newline
    let patched = if content.ends_with('\n') {
        format!("{patched}\n")
    } else {
        patched
    };

    std::fs::write(path, &patched)
        .map_err(|e| DepsError::ServicePatchFailed(format!("write {}: {e}", path.display())))?;

    info!(
        path = %path.display(),
        "deps: service file patched — ExecStart now includes -U"
    );
    Ok(())
}

// ── Systemd helpers ───────────────────────────────────────────────────────────

fn systemctl_daemon_reload() {
    if run_cmd("systemctl", &["daemon-reload"]) {
        info!("deps: systemd daemon-reload OK");
    } else {
        warn!("deps: systemctl daemon-reload failed (non-fatal)");
    }
}

fn enable_and_start() -> Result<(), DepsError> {
    // Enable so it starts on boot
    if !run_cmd("systemctl", &["enable", "zerotier-one"]) {
        warn!("deps: systemctl enable zerotier-one failed (non-fatal)");
    }

    // Start (or restart if already running with old config)
    let started = run_cmd("systemctl", &["restart", "zerotier-one"]);
    if !started {
        // Attempt plain start as fallback
        let ok = run_cmd("systemctl", &["start", "zerotier-one"]);
        if !ok {
            let stderr = capture_stderr("systemctl", &["status", "zerotier-one"]);
            let msg = format!("systemctl start/restart failed:\n{stderr}");
            error!("deps: {msg}");
            return Err(DepsError::ServiceStartFailed(msg));
        }
    }

    info!("deps: zerotier-one.service started");
    Ok(())
}

// ── Daemon readiness ──────────────────────────────────────────────────────────

/// Wait until the authtoken file appears, which signals the daemon is listening.
/// Authtoken is created by zerotier-one when it starts successfully.
fn wait_for_daemon(timeout_secs: u64) -> Result<(), DepsError> {
    // ZeroTier home directory candidates
    let candidates = zt_home_candidates();

    let deadline = Instant::now() + Duration::from_secs(timeout_secs);
    let poll = Duration::from_millis(500);

    info!("deps: waiting up to {timeout_secs}s for zerotier-one daemon…");

    while Instant::now() < deadline {
        // Check authtoken file existence (reliable readiness signal)
        if candidates
            .iter()
            .any(|dir| dir.join("authtoken.secret").exists())
        {
            return Ok(());
        }
        // Also accept: zerotier-cli info succeeds
        if run_cmd_quiet("zerotier-cli", &["info"]) {
            return Ok(());
        }
        thread::sleep(poll);
    }

    // Final attempt
    let status = capture_stderr("systemctl", &["status", "zerotier-one"]);
    error!("deps: zerotier-one not ready after {timeout_secs}s\n{status}");
    Err(DepsError::DaemonNotReady(timeout_secs))
}

fn zt_home_candidates() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/var/lib/zerotier-one"), // Linux (standard)
        PathBuf::from("/Library/Application Support/ZeroTier/One"), // macOS
    ]
}

// ── Version read ──────────────────────────────────────────────────────────────

fn read_version() -> Option<String> {
    let out = Command::new("zerotier-cli").arg("info").output().ok()?;
    if !out.status.success() {
        return None;
    }
    // "200 info <node-id> <version> ONLINE"
    let s = String::from_utf8_lossy(&out.stdout);
    let mut parts = s.split_whitespace();
    let code = parts.next()?;
    let _info = parts.next()?;
    let _id = parts.next()?;
    let version = parts.next()?;
    (code == "200").then(|| version.to_string())
}

// ── Command helpers ───────────────────────────────────────────────────────────

/// Run a command, return true if exit status is success.
fn run_cmd(cmd: &str, args: &[&str]) -> bool {
    Command::new(cmd)
        .args(args)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Run a command silently (no stdout/stderr), return true if success.
fn run_cmd_quiet(cmd: &str, args: &[&str]) -> bool {
    Command::new(cmd)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Capture stderr of a command (for diagnostics in error messages).
fn capture_stderr(cmd: &str, args: &[&str]) -> String {
    Command::new(cmd)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map(|o| String::from_utf8_lossy(&o.stderr).into_owned())
        .unwrap_or_default()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skip_deps_env_var() {
        // ZTNET_SKIP_DEPS=1 must make ensure() return Ok immediately
        // (safe to call in any environment, even without zerotier-one installed)
        std::env::set_var("ZTNET_SKIP_DEPS", "1");
        let result = ensure();
        std::env::remove_var("ZTNET_SKIP_DEPS");
        assert!(result.is_ok(), "ZTNET_SKIP_DEPS=1 must bypass all checks");
    }

    #[test]
    fn patch_idempotent() {
        let service = "\
[Unit]\nDescription=ZeroTier One\n\n\
[Service]\nExecStart=/usr/sbin/zerotier-one\nRestart=always\n\
[Install]\nWantedBy=multi-user.target\n";

        // First patch
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("zerotier-one.service");
        std::fs::write(&path, service).unwrap();
        patch_exec_start(&path).unwrap();

        let after = std::fs::read_to_string(&path).unwrap();
        assert!(after.contains("zerotier-one -U"), "should add -U flag");

        // Second patch must be idempotent
        patch_exec_start(&path).unwrap();
        let after2 = std::fs::read_to_string(&path).unwrap();
        let count = after2.matches("-U").count();
        assert_eq!(count, 1, "-U should appear exactly once");
    }

    #[test]
    fn patch_preserves_existing_flags() {
        let service = "\
[Service]\nExecStart=/usr/sbin/zerotier-one --some-flag\n";
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("zerotier-one.service");
        std::fs::write(&path, service).unwrap();
        patch_exec_start(&path).unwrap();

        let after = std::fs::read_to_string(&path).unwrap();
        assert!(after.contains("--some-flag -U"), "existing flags preserved");
    }

    #[test]
    fn patch_skips_when_already_patched() {
        let service = "[Service]\nExecStart=/usr/sbin/zerotier-one -U\n";
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("zerotier-one.service");
        std::fs::write(&path, service).unwrap();

        // Should return Ok without modifying
        patch_exec_start(&path).unwrap();
        let after = std::fs::read_to_string(&path).unwrap();
        assert_eq!(after, service, "file unchanged when already patched");
    }

    #[test]
    fn pm_detection_does_not_panic() {
        let _ = detect_pm();
    }

    #[test]
    fn zt_home_candidates_non_empty() {
        assert!(!zt_home_candidates().is_empty());
    }
}
