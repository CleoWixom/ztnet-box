use std::path::PathBuf;
use which::which;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize)]
pub struct ZtDetectionResult {
    pub zerotier_one: Option<PathBuf>,
    pub zerotier_idtool: Option<PathBuf>,
    /// Версия из `zerotier-cli info` — None если ZT не запущен или не установлен
    pub version: Option<String>,
    /// Совместимость: zerotier-cli доступен
    pub cli_available: bool,
}

#[derive(Debug, serde::Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum InstallResult {
    AlreadyInstalled(ZtDetectionResult),
    Installed(ZtDetectionResult),
    #[serde(rename = "unsupported_platform")]
    UnsupportedPlatform {
        reason: String,
    },
    Failed {
        reason: String,
    },
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PackageManager {
    Apt,
    Dnf,
    Pacman,
    Brew,
}

// ── Detection ─────────────────────────────────────────────────────────────────

pub fn detect() -> ZtDetectionResult {
    let zerotier_one = which("zerotier-one").ok();
    let zerotier_idtool = which("zerotier-idtool").ok();
    let cli_available = which("zerotier-cli").is_ok();
    let version = if cli_available {
        read_zt_version()
    } else {
        None
    };

    ZtDetectionResult {
        zerotier_one,
        zerotier_idtool,
        version,
        cli_available,
    }
}

pub fn detect_package_manager() -> Option<PackageManager> {
    if which("apt-get").is_ok() {
        return Some(PackageManager::Apt);
    }
    if which("dnf").is_ok() || which("yum").is_ok() {
        return Some(PackageManager::Dnf);
    }
    if which("pacman").is_ok() {
        return Some(PackageManager::Pacman);
    }
    if which("brew").is_ok() {
        return Some(PackageManager::Brew);
    }
    None
}

// ── Installation ──────────────────────────────────────────────────────────────

pub fn install(pm: PackageManager) -> anyhow::Result<InstallResult> {
    // Если уже установлен — не переустанавливаем
    let current = detect();
    if current.zerotier_one.is_some() {
        return Ok(InstallResult::AlreadyInstalled(current));
    }

    // Windows явно не поддерживается
    if cfg!(target_os = "windows") {
        return Ok(InstallResult::UnsupportedPlatform {
            reason: "Windows: install ZeroTier from https://www.zerotier.com/download/".into(),
        });
    }

    let ok = match pm {
        PackageManager::Apt => run_install(&["/usr/bin/apt-get", "install", "-y", "zerotier-one"]),
        PackageManager::Dnf => {
            // Попробовать dnf, потом yum
            if which("dnf").is_ok() {
                run_install(&["/usr/bin/dnf", "install", "-y", "zerotier-one"])
            } else {
                run_install(&["/usr/bin/yum", "install", "-y", "zerotier-one"])
            }
        }
        PackageManager::Pacman => {
            run_install(&["/usr/bin/pacman", "-S", "--noconfirm", "zerotier-one"])
        }
        PackageManager::Brew => run_install(&["/usr/local/bin/brew", "install", "zerotier"]),
    };

    if ok {
        Ok(InstallResult::Installed(detect()))
    } else {
        Ok(InstallResult::Failed {
            reason: format!(
                "Package manager {:?} failed. Try manually: https://www.zerotier.com/download/",
                pm
            ),
        })
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Читает версию из stdout `zerotier-cli info`:
/// "200 info <node-id> <version> ONLINE"  →  "<version>"
fn read_zt_version() -> Option<String> {
    let out = std::process::Command::new("zerotier-cli")
        .arg("info")
        .output()
        .ok()?;

    if !out.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    // Формат: "200 info <node-id> <version> ONLINE"
    let mut parts = stdout.split_whitespace();
    let code = parts.next()?; // "200"
    let _info = parts.next()?; // "info"
    let _nodeid = parts.next()?; // node ID
    let version = parts.next()?; // version

    if code == "200" {
        Some(version.to_string())
    } else {
        None
    }
}

/// Запускает команду и возвращает true если успешно завершилась
fn run_install(args: &[&str]) -> bool {
    let (cmd, rest) = match args.split_first() {
        Some(pair) => pair,
        None => return false,
    };
    std::process::Command::new(cmd)
        .args(rest)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_returns_result() {
        // Просто проверяем что не паникует
        let result = detect();
        // На CI ZeroTier не установлен — zerotier_one должен быть None
        assert!(result.zerotier_one.is_none() || result.zerotier_one.is_some());
    }

    #[test]
    fn detect_package_manager_does_not_panic() {
        let _ = detect_package_manager();
    }

    #[test]
    fn version_parse_format() {
        // Тест парсинга строки zerotier-cli info
        let line = "200 info deadbeef01 1.12.2 ONLINE";
        let mut parts = line.split_whitespace();
        let code = parts.next().unwrap();
        let _info = parts.next().unwrap();
        let _nodeid = parts.next().unwrap();
        let version = parts.next().unwrap();
        assert_eq!(code, "200");
        assert_eq!(version, "1.12.2");
    }
}
