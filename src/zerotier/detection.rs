use std::path::PathBuf;
use which::which;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ZtDetectionResult {
    pub zerotier_one: Option<PathBuf>,
    pub zerotier_idtool: Option<PathBuf>,
    pub version: Option<String>,
}

#[derive(Debug, serde::Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum InstallResult {
    AlreadyInstalled(ZtDetectionResult),
    Installed(ZtDetectionResult),
    UnsupportedPlatform { reason: String },
    Failed { reason: String },
}

#[derive(Debug, Clone, Copy)]
pub enum PackageManager {
    Apt,
    Dnf,
    Pacman,
    Brew,
}

pub fn detect() -> ZtDetectionResult {
    let zerotier_one = which("zerotier-one").ok();
    let zerotier_idtool = which("zerotier-idtool").ok();
    let version = zerotier_one.as_ref().and_then(|_| read_zt_version());
    ZtDetectionResult {
        zerotier_one,
        zerotier_idtool,
        version,
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

pub fn install(pm: PackageManager) -> anyhow::Result<InstallResult> {
    let result = detect();
    if result.zerotier_one.is_some() {
        return Ok(InstallResult::AlreadyInstalled(result));
    }

    let status = match pm {
        PackageManager::Apt => std::process::Command::new("/usr/bin/apt-get")
            .args(["install", "-y", "zerotier-one"])
            .status()?,
        PackageManager::Dnf => std::process::Command::new("/usr/bin/dnf")
            .args(["install", "-y", "zerotier-one"])
            .status()?,
        PackageManager::Pacman => std::process::Command::new("/usr/bin/pacman")
            .args(["-S", "--noconfirm", "zerotier-one"])
            .status()?,
        PackageManager::Brew => std::process::Command::new("/usr/local/bin/brew")
            .args(["install", "zerotier"])
            .status()?,
    };

    if status.success() {
        Ok(InstallResult::Installed(detect()))
    } else {
        Ok(InstallResult::Failed {
            reason: format!("Package manager exited with {status}"),
        })
    }
}

fn read_zt_version() -> Option<String> {
    let out = std::process::Command::new("zerotier-cli")
        .arg("info")
        .output()
        .ok()?;
    // Output: "200 info <node-id> <version> ONLINE"
    let s = String::from_utf8_lossy(&out.stdout);
    s.split_whitespace().nth(3).map(|v| v.to_string())
}
