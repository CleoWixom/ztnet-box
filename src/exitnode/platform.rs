use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PlatformSupport {
    pub supported: bool,
    pub os: String,
    pub reason: Option<String>,
}

pub fn check() -> PlatformSupport {
    #[cfg(target_os = "linux")]
    {
        PlatformSupport {
            supported: true,
            os: "linux".into(),
            reason: None,
        }
    }
    #[cfg(target_os = "macos")]
    {
        PlatformSupport {
            supported: false,
            os: "macos".into(),
            reason: Some(
                "Exit Node requires Linux (iptables/nftables). macOS is not supported.".into(),
            ),
        }
    }
    #[cfg(target_os = "windows")]
    {
        PlatformSupport {
            supported: false,
            os: "windows".into(),
            reason: Some("Exit Node requires Linux. Windows is not supported.".into()),
        }
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        PlatformSupport {
            supported: false,
            os: std::env::consts::OS.to_string(),
            reason: Some(format!(
                "Exit Node is not supported on {}",
                std::env::consts::OS
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_returns_result() {
        let p = check();
        assert!(!p.os.is_empty());
        // On Linux (CI) supported=true, elsewhere false
        #[cfg(target_os = "linux")]
        assert!(p.supported);
        #[cfg(not(target_os = "linux"))]
        assert!(!p.supported);
    }
}
