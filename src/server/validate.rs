/// Input validation helpers for API path parameters and body fields.
use crate::server::error::ApiError;

/// ZeroTier network ID: exactly 16 hex characters, e.g. "8056c2e21c000001"
pub fn network_id(id: &str) -> Result<(), ApiError> {
    if id.len() == 16 && id.chars().all(|c| c.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(ApiError::InvalidInput(format!(
            "invalid network_id '{}': must be exactly 16 hex characters",
            id
        )))
    }
}

/// ZeroTier node/member ID: exactly 10 hex characters, e.g. "a29a9a9a9a"
pub fn node_id(id: &str) -> Result<(), ApiError> {
    if id.len() == 10 && id.chars().all(|c| c.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(ApiError::InvalidInput(format!(
            "invalid node_id '{}': must be exactly 10 hex characters",
            id
        )))
    }
}

/// ZeroTier moon world_id: 1–16 hex characters
pub fn world_id(id: &str) -> Result<(), ApiError> {
    if !id.is_empty() && id.len() <= 16 && id.chars().all(|c| c.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(ApiError::InvalidInput(format!(
            "invalid world_id '{}': must be 1–16 hex characters",
            id
        )))
    }
}

/// IPv4/IPv6 address
pub fn ip_addr(s: &str) -> Result<std::net::IpAddr, ApiError> {
    s.parse::<std::net::IpAddr>()
        .map_err(|_| ApiError::InvalidInput(format!("invalid IP address: '{}'", s)))
}

/// CIDR notation, e.g. "192.168.1.0/24"
pub fn cidr(s: &str) -> Result<(), ApiError> {
    let parts: Vec<&str> = s.splitn(2, '/').collect();
    if parts.len() != 2 {
        return Err(ApiError::InvalidInput(format!(
            "invalid CIDR '{}': expected addr/prefix",
            s
        )));
    }
    ip_addr(parts[0])?;
    let prefix: u8 = parts[1]
        .parse()
        .map_err(|_| ApiError::InvalidInput(format!("invalid CIDR prefix in '{}'", s)))?;
    let max_prefix = if parts[0].contains(':') { 128 } else { 32 };
    if prefix > max_prefix {
        return Err(ApiError::InvalidInput(format!(
            "CIDR prefix {} out of range (max {})",
            prefix, max_prefix
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_network_id() {
        assert!(network_id("8056c2e21c000001").is_ok());
    }
    #[test]
    fn short_network_id_rejected() {
        assert!(network_id("8056c2e2").is_err());
    }
    #[test]
    fn non_hex_network_id_rejected() {
        assert!(network_id("8056c2e21c00000g").is_err());
    }

    #[test]
    fn valid_node_id() {
        assert!(node_id("a29a9a9a9a").is_ok());
    }
    #[test]
    fn short_node_id_rejected() {
        assert!(node_id("a29a").is_err());
    }

    #[test]
    fn valid_ip() {
        assert!(ip_addr("192.168.1.1").is_ok());
        assert!(ip_addr("::1").is_ok());
    }
    #[test]
    fn invalid_ip_rejected() {
        assert!(ip_addr("999.999.999.999").is_err());
        assert!(ip_addr("not-an-ip").is_err());
    }

    #[test]
    fn valid_cidr() {
        assert!(cidr("10.0.0.0/8").is_ok());
        assert!(cidr("fd00::/8").is_ok());
    }
    #[test]
    fn invalid_cidr_rejected() {
        assert!(cidr("10.0.0.0").is_err());
        assert!(cidr("10.0.0.0/33").is_err());
        assert!(cidr("bad/8").is_err());
    }
}
