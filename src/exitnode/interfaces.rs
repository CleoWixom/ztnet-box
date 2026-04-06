use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct NetworkInterface {
    pub name: String,
    pub addresses: Vec<String>,
    pub is_zerotier: bool,
}

pub fn list_interfaces() -> Result<Vec<NetworkInterface>, String> {
    #[cfg(unix)]
    return list_unix();
    #[cfg(not(unix))]
    return Ok(Vec::new());
}

#[cfg(unix)]
fn list_unix() -> Result<Vec<NetworkInterface>, String> {
    use std::collections::HashMap;

    let mut iface_addrs: HashMap<String, Vec<String>> = HashMap::new();

    // Seed interface names from /proc/net/dev (always available on Linux)
    if let Ok(content) = std::fs::read_to_string("/proc/net/dev") {
        for line in content.lines().skip(2) {
            let name = line.split(':').next().unwrap_or("").trim().to_string();
            if !name.is_empty() {
                iface_addrs.entry(name).or_default();
            }
        }
    }

    // Populate addresses via getifaddrs
    match nix::ifaddrs::getifaddrs() {
        Ok(iter) => {
            for ifaddr in iter {
                let name = ifaddr.interface_name.clone();
                // Try IPv4 first
                if let Some(addr) = ifaddr.address {
                    // SockaddrStorage: try as SockaddrIn (IPv4) or SockaddrIn6 (IPv6)
                    if let Some(inet) = addr.as_sockaddr_in() {
                        let ip = inet.ip();
                        iface_addrs.entry(name).or_default().push(ip.to_string());
                    } else if let Some(inet6) = addr.as_sockaddr_in6() {
                        let ip = inet6.ip();
                        iface_addrs.entry(name).or_default().push(ip.to_string());
                    }
                }
            }
        }
        Err(e) => tracing::warn!(error = %e, "getifaddrs failed, using names only"),
    }

    let mut result: Vec<NetworkInterface> = iface_addrs
        .into_iter()
        .map(|(name, addresses)| {
            let is_zerotier = is_zerotier_iface(&name);
            NetworkInterface {
                name,
                addresses,
                is_zerotier,
            }
        })
        .collect();

    result.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(result)
}

fn is_zerotier_iface(name: &str) -> bool {
    name.starts_with("zt") && name.len() > 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_does_not_panic() {
        let _ = list_interfaces();
    }

    #[test]
    fn zerotier_detection() {
        assert!(is_zerotier_iface("zt3jh2bk1a"));
        assert!(!is_zerotier_iface("eth0"));
        assert!(!is_zerotier_iface("lo"));
        assert!(!is_zerotier_iface("zt")); // too short
    }
}
