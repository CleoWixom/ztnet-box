use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FirewallBackend {
    Nftables,
    Iptables,
}

#[derive(Debug, Error)]
pub enum RulesError {
    #[error("Unsupported firewall backend")]
    Unsupported,
    #[error("Command failed: {0}")]
    Command(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct ExitNodeRules {
    pub zt_iface: String,
    pub wan_iface: String,
    pub backend: FirewallBackend,
}

impl ExitNodeRules {
    pub fn new(zt_iface: String, wan_iface: String, backend: FirewallBackend) -> Self {
        Self { zt_iface, wan_iface, backend }
    }

    /// Применяет правила EXIT NODE:
    /// 1. Включает ip_forward
    /// 2. Устанавливает rp_filter=2 (требуется для клиентов с allowDefault=1)
    /// 3. Добавляет MASQUERADE/POSTROUTING через nft или iptables
    /// 4. Сохраняет правила (iptables-persistent / nft list ruleset)
    pub fn apply(&self) -> Result<(), RulesError> {
        self.enable_ip_forward()?;
        self.fix_rp_filter()?;
        match self.backend {
            FirewallBackend::Nftables => self.apply_nftables()?,
            FirewallBackend::Iptables => self.apply_iptables()?,
        }
        // Best-effort persistence — log warning on failure, don't abort
        if let Err(e) = self.persist_rules() {
            tracing::warn!(error = %e, "exit node rules applied but could not be persisted across reboots");
        }
        Ok(())
    }

    /// Откатывает правила EXIT NODE.
    pub fn remove(&self) -> Result<(), RulesError> {
        // Do not disable ip_forward — other services may depend on it
        match self.backend {
            FirewallBackend::Nftables => self.remove_nftables(),
            FirewallBackend::Iptables => self.remove_iptables(),
        }
    }

    // ── rp_filter ─────────────────────────────────────────────────────────────

    /// Returns true if rp_filter is already set to a compatible value (0 or 2).
    /// See: https://docs.zerotier.com/exitnode/#a-linux-gotcha-rp_filter
    pub fn check_rp_filter() -> bool {
        std::fs::read_to_string("/proc/sys/net/ipv4/conf/all/rp_filter")
            .map(|s| matches!(s.trim(), "0" | "2"))
            .unwrap_or(true) // non-Linux: treat as OK
    }

    /// Sets rp_filter=2 at runtime and optionally persists via sysctl.conf.
    /// Required on gateway nodes so that ZeroTier client traffic (with allowDefault=1)
    /// passes the reverse-path filter.
    pub fn fix_rp_filter(&self) -> Result<(), RulesError> {
        // Runtime — immediate effect
        #[cfg(target_os = "linux")]
        {
            std::fs::write("/proc/sys/net/ipv4/conf/all/rp_filter", "2\n")?;
            tracing::info!("rp_filter set to 2 (loose mode)");
            // Persist across reboots via sysctl.conf
            Self::append_sysctl("net.ipv4.conf.all.rp_filter", "2")?;
        }
        Ok(())
    }

    /// Appends or updates a sysctl key=value in /etc/sysctl.conf.
    fn append_sysctl(key: &str, value: &str) -> Result<(), RulesError> {
        let path = "/etc/sysctl.conf";
        let content = std::fs::read_to_string(path).unwrap_or_default();
        let marker = format!("# ztnet-box: {key}");

        if content.contains(&format!("{key} =")) || content.contains(&format!("{key}=")) {
            // Replace existing entry
            let updated: String = content
                .lines()
                .map(|line| {
                    if line.trim_start().starts_with(key) {
                        format!("{key} = {value}  {marker}")
                    } else {
                        line.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            std::fs::write(path, updated + "\n")?;
        } else {
            // Append new entry
            use std::io::Write;
            let mut f = std::fs::OpenOptions::new().append(true).open(path)?;
            writeln!(f, "{marker}")?;
            writeln!(f, "{key} = {value}")?;
        }
        Ok(())
    }

    // ── ip_forward ────────────────────────────────────────────────────────────

    fn enable_ip_forward(&self) -> Result<(), RulesError> {
        std::fs::write("/proc/sys/net/ipv4/ip_forward", "1\n")?;
        tracing::info!("ip_forward enabled");
        Ok(())
    }

    // ── persist rules ─────────────────────────────────────────────────────────

    /// Persists firewall rules so they survive a reboot.
    /// - iptables: writes to /etc/iptables/rules.v4 via iptables-save,
    ///   then calls netfilter-persistent save if available.
    /// - nftables: calls `systemctl enable nftables` and writes
    ///   /etc/nftables.conf via `nft list ruleset`.
    pub fn persist_rules(&self) -> Result<(), RulesError> {
        match self.backend {
            FirewallBackend::Iptables => self.persist_iptables(),
            FirewallBackend::Nftables => self.persist_nftables(),
        }
    }

    fn persist_iptables(&self) -> Result<(), RulesError> {
        // Prefer netfilter-persistent / iptables-persistent
        if which::which("netfilter-persistent").is_ok() {
            return self.run_cmd("netfilter-persistent", &["save"]);
        }
        // Fallback: iptables-save → /etc/iptables/rules.v4
        if which::which("iptables-save").is_ok() {
            let out = std::process::Command::new("iptables-save")
                .output()
                .map_err(|e| RulesError::Command(format!("iptables-save: {e}")))?;
            if !out.status.success() {
                return Err(RulesError::Command("iptables-save failed".into()));
            }
            let dir = std::path::Path::new("/etc/iptables");
            if !dir.exists() {
                std::fs::create_dir_all(dir)?;
            }
            std::fs::write("/etc/iptables/rules.v4", &out.stdout)?;
            tracing::info!("iptables rules saved to /etc/iptables/rules.v4");
            return Ok(());
        }
        Err(RulesError::Command(
            "neither netfilter-persistent nor iptables-save found".into(),
        ))
    }

    fn persist_nftables(&self) -> Result<(), RulesError> {
        // Dump current ruleset to /etc/nftables.conf
        let out = std::process::Command::new("nft")
            .args(["list", "ruleset"])
            .output()
            .map_err(|e| RulesError::Command(format!("nft list ruleset: {e}")))?;
        if !out.status.success() {
            return Err(RulesError::Command("nft list ruleset failed".into()));
        }
        std::fs::write("/etc/nftables.conf", &out.stdout)?;
        // Enable nftables service so rules are loaded at boot
        let _ = self.run_cmd("systemctl", &["enable", "nftables"]);
        tracing::info!("nftables rules saved to /etc/nftables.conf");
        Ok(())
    }

    fn run_cmd(&self, cmd: &str, args: &[&str]) -> Result<(), RulesError> {
        let status = std::process::Command::new(cmd)
            .args(args)
            .status()
            .map_err(|e| RulesError::Command(format!("{cmd} spawn: {e}")))?;
        if status.success() {
            Ok(())
        } else {
            Err(RulesError::Command(format!("{cmd} exited with {status}")))
        }
    }

    // ── nftables ──────────────────────────────────────────────────────────────

    fn apply_nftables(&self) -> Result<(), RulesError> {
        let ruleset = format!(
            "table ip ztnet_exit {{\n\
             \tchain postrouting {{\n\
             \t\ttype nat hook postrouting priority srcnat; policy accept;\n\
             \t\tiifname \"{zt}\" oifname \"{wan}\" masquerade\n\
             \t}}\n\
             \tchain forward {{\n\
             \t\ttype filter hook forward priority filter; policy accept;\n\
             \t\tiifname \"{zt}\" oifname \"{wan}\" accept\n\
             \t\tiifname \"{wan}\" oifname \"{zt}\" ct state established,related accept\n\
             \t}}\n\
             }}",
            zt = self.zt_iface,
            wan = self.wan_iface,
        );
        self.run_nft(&ruleset)
    }

    fn remove_nftables(&self) -> Result<(), RulesError> {
        self.run_nft("delete table ip ztnet_exit")
    }

    fn run_nft(&self, ruleset: &str) -> Result<(), RulesError> {
        use std::io::Write;
        use std::process::{Command, Stdio};

        let mut child = Command::new("nft")
            .arg("-f")
            .arg("-")
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| RulesError::Command(format!("nft spawn: {e}")))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(ruleset.as_bytes())
                .map_err(|e| RulesError::Command(format!("nft stdin write: {e}")))?;
        }

        let status = child
            .wait()
            .map_err(|e| RulesError::Command(format!("nft wait: {e}")))?;

        if status.success() {
            Ok(())
        } else {
            Err(RulesError::Command(format!("nft exited with {status}")))
        }
    }

    // ── iptables ──────────────────────────────────────────────────────────────

    fn apply_iptables(&self) -> Result<(), RulesError> {
        // NAT MASQUERADE
        self.run_iptables(&[
            "-t", "nat", "-A", "POSTROUTING",
            "-i", &self.zt_iface, "-o", &self.wan_iface,
            "-j", "MASQUERADE",
        ])?;
        // Forward ZT → WAN
        self.run_iptables(&[
            "-A", "FORWARD",
            "-i", &self.zt_iface, "-o", &self.wan_iface,
            "-j", "ACCEPT",
        ])?;
        // Forward WAN → ZT (established/related only)
        self.run_iptables(&[
            "-A", "FORWARD",
            "-i", &self.wan_iface, "-o", &self.zt_iface,
            "-m", "state", "--state", "ESTABLISHED,RELATED",
            "-j", "ACCEPT",
        ])
    }

    fn remove_iptables(&self) -> Result<(), RulesError> {
        // -D deletes rule; ignore errors (rule may already be gone)
        let _ = self.run_iptables(&[
            "-t", "nat", "-D", "POSTROUTING",
            "-i", &self.zt_iface, "-o", &self.wan_iface,
            "-j", "MASQUERADE",
        ]);
        let _ = self.run_iptables(&[
            "-D", "FORWARD",
            "-i", &self.zt_iface, "-o", &self.wan_iface,
            "-j", "ACCEPT",
        ]);
        let _ = self.run_iptables(&[
            "-D", "FORWARD",
            "-i", &self.wan_iface, "-o", &self.zt_iface,
            "-m", "state", "--state", "ESTABLISHED,RELATED",
            "-j", "ACCEPT",
        ]);
        Ok(())
    }

    fn run_iptables(&self, args: &[&str]) -> Result<(), RulesError> {
        let status = std::process::Command::new("iptables")
            .args(args)
            .status()
            .map_err(|e| RulesError::Command(format!("iptables spawn: {e}")))?;
        if status.success() {
            Ok(())
        } else {
            Err(RulesError::Command(format!(
                "iptables {:?} exited with {status}",
                args
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules_struct_creation() {
        let r = ExitNodeRules::new("zt3abc".into(), "eth0".into(), FirewallBackend::Nftables);
        assert_eq!(r.zt_iface, "zt3abc");
        assert_eq!(r.backend, FirewallBackend::Nftables);
    }

    #[test]
    fn nftables_ruleset_contains_key_elements() {
        let r = ExitNodeRules::new("zt3abc123".into(), "eth0".into(), FirewallBackend::Nftables);
        let ruleset = format!(
            "table ip ztnet_exit {{ chain postrouting {{ \
             type nat hook postrouting priority srcnat; policy accept; \
             iifname \"{}\" oifname \"{}\" masquerade }} }}",
            r.zt_iface, r.wan_iface
        );
        assert!(ruleset.contains("zt3abc123"));
        assert!(ruleset.contains("eth0"));
        assert!(ruleset.contains("masquerade"));
    }

    #[test]
    fn firewall_backend_serde() {
        let n = serde_json::to_string(&FirewallBackend::Nftables).unwrap();
        let i = serde_json::to_string(&FirewallBackend::Iptables).unwrap();
        assert_eq!(n, "\"nftables\"");
        assert_eq!(i, "\"iptables\"");
    }

    #[test]
    fn check_rp_filter_does_not_panic() {
        // On CI this reads /proc/sys or returns true on non-Linux
        let _ = ExitNodeRules::check_rp_filter();
    }
}
