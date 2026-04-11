use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct PlatformInfo {
    pub supported: bool,
    pub os: &'static str,
    pub reason: Option<&'static str>,
}

pub fn check() -> PlatformInfo {
    #[cfg(target_os = "linux")]
    return PlatformInfo {
        supported: true,
        os: "linux",
        reason: None,
    };

    #[cfg(not(target_os = "linux"))]
    return PlatformInfo {
        supported: false,
        os: std::env::consts::OS,
        reason: Some("Layer 2 Bridge requires Linux (systemd-networkd + ip link)"),
    };
}
