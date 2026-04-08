//! ZeroTier local.conf — per-node configuration file.
//! See: https://docs.zerotier.com/config/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocalConf {
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub physical: HashMap<String, PhysicalPathConfig>,

    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub virtual_: HashMap<String, VirtualNodeConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<LocalSettings>,
}

/// Physical path configuration: blacklist a CIDR from being used as a ZT path.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PhysicalPathConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blacklist: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trust: Option<String>,
}

/// Virtual node config: always/never try to reach a specific ZT node directly.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VirtualNodeConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub try_: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blacklist: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LocalSettings {
    /// Primary UDP port (0 = auto / OS-assigned). Default: 9993
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_port: Option<u16>,
    /// Secondary UDP port (0 = disabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary_port: Option<u16>,
    /// Tertiary UDP port (0 = disabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tertiary_port: Option<u16>,
    /// Enable UPnP/NAT-PMP port mapping (default: true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port_mapping_enabled: Option<bool>,
    /// Force all traffic through a TCP relay (impacts performance)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_tcp_relay: Option<bool>,
    /// Allow TCP fallback relay when UDP fails (default: true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_tcp_fallback_relay: Option<bool>,
    /// Custom TCP relay endpoint: "ip/port" (e.g. "1.2.3.4/443")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcp_fallback_relay: Option<String>,
    /// Interface name prefixes to ignore (e.g. ["docker", "virbr"])
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interface_prefix_blacklist: Option<Vec<String>>,
    /// CIDRs or IPs allowed to access the management API (default: ["127.0.0.1", "::1"])
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_management_from: Option<Vec<String>>,
    /// Specific IPs/interfaces to bind to (empty = bind all)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<Vec<String>>,
    /// Enable ZeroTier metrics endpoint (ZT >= 1.16.1, default: false)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_metrics: Option<bool>,
}

/// Per-network local configuration (<network-id>.local.conf)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NetworkLocalConf {
    /// Allow ZeroTier to manage IPv4/IPv6 routes (default: true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_managed: Option<bool>,
    /// Allow assignment of global IPs from ZeroTier (default: false)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_global: Option<bool>,
    /// Allow ZeroTier to set a default route — required for full-tunnel VPN / Exit Node clients
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_default: Option<bool>,
    /// Allow ZeroTier to manage DNS (default: false)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_dns: Option<bool>,
}

// ── Paths ─────────────────────────────────────────────────────────────────────

/// Returns the platform-specific ZeroTier home directory.
pub fn zerotier_home() -> PathBuf {
    #[cfg(target_os = "linux")]
    return PathBuf::from("/var/lib/zerotier-one");
    #[cfg(target_os = "macos")]
    return PathBuf::from("/Library/Application Support/ZeroTier/One");
    #[cfg(target_os = "windows")]
    return PathBuf::from("C:\\ProgramData\\ZeroTier\\One");
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    return PathBuf::from("/var/lib/zerotier-one");
}

pub fn local_conf_path() -> PathBuf {
    zerotier_home().join("local.conf")
}

pub fn network_local_conf_path(network_id: &str) -> PathBuf {
    zerotier_home().join(format!("{network_id}.local.conf"))
}

// ── Read / Write ──────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum LocalConfError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Read local.conf, returning empty defaults if the file doesn't exist.
pub fn read(path: &Path) -> Result<LocalConf, LocalConfError> {
    match std::fs::read_to_string(path) {
        Ok(s) => Ok(serde_json::from_str(&s)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(LocalConf::default()),
        Err(e) => Err(e.into()),
    }
}

/// Write local.conf as pretty-printed JSON.
pub fn write(path: &Path, conf: &LocalConf) -> Result<(), LocalConfError> {
    let json = serde_json::to_string_pretty(conf)?;
    std::fs::write(path, json + "\n")?;
    Ok(())
}

/// Read <network-id>.local.conf, returning empty defaults if missing.
pub fn read_network(network_id: &str) -> Result<NetworkLocalConf, LocalConfError> {
    let path = network_local_conf_path(network_id);
    match std::fs::read_to_string(&path) {
        Ok(s) => Ok(serde_json::from_str(&s)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(NetworkLocalConf::default()),
        Err(e) => Err(e.into()),
    }
}

/// Write <network-id>.local.conf.
pub fn write_network(network_id: &str, conf: &NetworkLocalConf) -> Result<(), LocalConfError> {
    let path = network_local_conf_path(network_id);
    let json = serde_json::to_string_pretty(conf)?;
    std::fs::write(path, json + "\n")?;
    Ok(())
}

// ── Conflict validation ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct ValidationWarning {
    pub field: String,
    pub message: String,
}

/// Validate LocalSettings for known bad combinations.
pub fn validate_settings(s: &LocalSettings) -> Vec<ValidationWarning> {
    let mut warnings = Vec::new();

    if s.force_tcp_relay == Some(true) && s.port_mapping_enabled == Some(true) {
        warnings.push(ValidationWarning {
            field: "forceTcpRelay".into(),
            message: "forceTcpRelay=true makes portMappingEnabled irrelevant; consider disabling portMapping to reduce noise".into(),
        });
    }
    if let (Some(p), Some(s_port)) = (s.primary_port, s.secondary_port) {
        if p != 0 && s_port != 0 && p == s_port {
            warnings.push(ValidationWarning {
                field: "secondaryPort".into(),
                message: "primaryPort and secondaryPort must differ".into(),
            });
        }
    }
    if let Some(ref blacklist) = s.interface_prefix_blacklist {
        if blacklist.iter().any(|p| p.starts_with("zt")) {
            warnings.push(ValidationWarning {
                field: "interfacePrefixBlacklist".into(),
                message: "Blacklisting 'zt*' prefixes will prevent ZeroTier from using its own interfaces".into(),
            });
        }
    }
    if let Some(ref allowed) = s.allow_management_from {
        let has_public = allowed.iter().any(|a| {
            !a.starts_with("127.") && !a.starts_with("::1") && a != "localhost"
        });
        if has_public {
            warnings.push(ValidationWarning {
                field: "allowManagementFrom".into(),
                message: "Management API is accessible from non-loopback addresses — ensure network-level access control".into(),
            });
        }
    }
    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_empty_local_conf() {
        let conf = LocalConf::default();
        let json = serde_json::to_string_pretty(&conf).unwrap();
        let back: LocalConf = serde_json::from_str(&json).unwrap();
        assert!(back.physical.is_empty());
        assert!(back.settings.is_none());
    }

    #[test]
    fn roundtrip_network_local_conf() {
        let conf = NetworkLocalConf {
            allow_managed: Some(true),
            allow_default: Some(true),
            allow_global: Some(false),
            allow_dns: None,
        };
        let json = serde_json::to_string_pretty(&conf).unwrap();
        assert!(json.contains("allowManaged"));
        assert!(json.contains("allowDefault"));
        assert!(!json.contains("allowDns")); // skip_serializing_if None
    }

    #[test]
    fn validate_force_tcp_relay_conflict() {
        let s = LocalSettings {
            force_tcp_relay: Some(true),
            port_mapping_enabled: Some(true),
            ..Default::default()
        };
        let w = validate_settings(&s);
        assert!(w.iter().any(|w| w.field == "forceTcpRelay"));
    }

    #[test]
    fn validate_same_ports() {
        let s = LocalSettings {
            primary_port: Some(9993),
            secondary_port: Some(9993),
            ..Default::default()
        };
        let w = validate_settings(&s);
        assert!(w.iter().any(|w| w.field == "secondaryPort"));
    }

    #[test]
    fn validate_zt_blacklist_warning() {
        let s = LocalSettings {
            interface_prefix_blacklist: Some(vec!["zt".into(), "docker".into()]),
            ..Default::default()
        };
        let w = validate_settings(&s);
        assert!(w.iter().any(|w| w.field == "interfacePrefixBlacklist"));
    }

    #[test]
    fn no_warnings_for_clean_settings() {
        let s = LocalSettings {
            primary_port: Some(9993),
            port_mapping_enabled: Some(true),
            ..Default::default()
        };
        assert!(validate_settings(&s).is_empty());
    }

    #[test]
    fn local_conf_path_is_absolute() {
        assert!(local_conf_path().is_absolute());
    }
}
