//! Conflict detection for Physical Network Routing.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ConflictReport {
    pub conflicts: Vec<Conflict>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Conflict {
    pub severity: ConflictSeverity,
    pub message: String,
    pub field: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ConflictSeverity {
    Error,
    Warning,
}

pub fn check(
    phy_subnet: &str,
    zt_subnet: Option<&str>,
    exitnode_enabled: bool,
    bridge_enabled: bool,
) -> ConflictReport {
    let mut conflicts = Vec::new();
    let warnings = Vec::new();

    if exitnode_enabled {
        conflicts.push(Conflict {
            severity: ConflictSeverity::Warning,
            message: "Exit Node is also active. iptables rules may conflict — \
                       review POSTROUTING/FORWARD chains manually."
                .into(),
            field: None,
        });
    }

    if bridge_enabled {
        conflicts.push(Conflict {
            severity: ConflictSeverity::Error,
            message:  "Layer 2 Bridge is active. Cannot use Physical Routing and L2 Bridge simultaneously.".into(),
            field: None,
        });
    }

    // Check for subnet overlap between ZT and physical subnets
    if let Some(zt) = zt_subnet {
        if subnets_overlap(phy_subnet, zt) {
            conflicts.push(Conflict {
                severity: ConflictSeverity::Warning,
                message: format!(
                    "Physical subnet {phy_subnet} overlaps with ZeroTier subnet {zt}. \
                     This may cause routing loops. Use the /23 trick for the managed route."
                ),
                field: Some("phy_subnet".into()),
            });
        }
    }

    ConflictReport {
        conflicts,
        warnings,
    }
}

/// Very basic overlap check: compare the network addresses
fn subnets_overlap(a: &str, b: &str) -> bool {
    // Extract base addresses (before /) and compare first two octets
    let base_a = a
        .split('/')
        .next()
        .unwrap_or("")
        .split('.')
        .take(2)
        .collect::<Vec<_>>()
        .join(".");
    let base_b = b
        .split('/')
        .next()
        .unwrap_or("")
        .split('.')
        .take(2)
        .collect::<Vec<_>>()
        .join(".");
    !base_a.is_empty() && base_a == base_b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_prefix_overlaps() {
        assert!(subnets_overlap("192.168.1.0/24", "192.168.2.0/24"));
    }

    #[test]
    fn different_prefix_no_overlap() {
        assert!(!subnets_overlap("10.0.0.0/8", "192.168.1.0/24"));
    }

    #[test]
    fn no_conflict_clean_state() {
        let r = check("192.168.1.0/24", Some("172.27.0.0/16"), false, false);
        assert!(r.conflicts.is_empty());
    }

    #[test]
    fn exitnode_conflict_is_warning() {
        let r = check("192.168.1.0/24", None, true, false);
        assert_eq!(r.conflicts.len(), 1);
        assert!(matches!(r.conflicts[0].severity, ConflictSeverity::Warning));
    }

    #[test]
    fn bridge_conflict_is_error() {
        let r = check("192.168.1.0/24", None, false, true);
        assert_eq!(r.conflicts.len(), 1);
        assert!(matches!(r.conflicts[0].severity, ConflictSeverity::Error));
    }
}
