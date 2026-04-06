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
        Self {
            zt_iface,
            wan_iface,
            backend,
        }
    }

    /// Применяет правила EXIT NODE:
    /// 1. Включает ip_forward
    /// 2. Добавляет MASQUERADE/POSTROUTING через nft или iptables
    pub fn apply(&self) -> Result<(), RulesError> {
        self.enable_ip_forward()?;
        match self.backend {
            FirewallBackend::Nftables => self.apply_nftables(),
            FirewallBackend::Iptables => self.apply_iptables(),
        }
    }

    /// Откатывает правила EXIT NODE.
    pub fn remove(&self) -> Result<(), RulesError> {
        // Не выключаем ip_forward — могут быть другие пользователи
        match self.backend {
            FirewallBackend::Nftables => self.remove_nftables(),
            FirewallBackend::Iptables => self.remove_iptables(),
        }
    }

    // ── ip_forward ────────────────────────────────────────────────────────────

    fn enable_ip_forward(&self) -> Result<(), RulesError> {
        std::fs::write("/proc/sys/net/ipv4/ip_forward", "1\n")?;
        tracing::info!("ip_forward enabled");
        Ok(())
    }

    // ── nftables ──────────────────────────────────────────────────────────────

    fn apply_nftables(&self) -> Result<(), RulesError> {
        // Создаём table + chain если нет, затем добавляем masquerade rule
        let ruleset = format!(
            "table ip ztnet_exit {{\n\
             \tchain postrouting {{\n\
             \t\ttype nat hook postrouting priority srcnat; policy accept;\n\
             \t\tiifname \"{zt}\" oifname \"{wan}\" masquerade\n\
             \t}}\n\
             }}",
            zt = self.zt_iface,
            wan = self.wan_iface,
        );
        self.run_nft(&ruleset)
    }

    fn remove_nftables(&self) -> Result<(), RulesError> {
        // Удаляем всю нашу table целиком
        let cmd = "delete table ip ztnet_exit";
        self.run_nft(cmd)
    }

    /// Запускает `nft -f -` подавая ruleset через stdin (без shell).
    fn run_nft(&self, ruleset: &str) -> Result<(), RulesError> {
        use std::io::Write;
        use std::process::{Command, Stdio};

        let mut child = Command::new("nft")
            .arg("-f")
            .arg("-")
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| RulesError::Command(format!("nft spawn: {e}")))?;

        if let Some(stdin) = child.stdin.take() {
            let mut stdin = stdin;
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
        self.run_iptables(&[
            "-t",
            "nat",
            "-A",
            "POSTROUTING",
            "-i",
            &self.zt_iface,
            "-o",
            &self.wan_iface,
            "-j",
            "MASQUERADE",
        ])?;
        self.run_iptables(&[
            "-A",
            "FORWARD",
            "-i",
            &self.zt_iface,
            "-o",
            &self.wan_iface,
            "-j",
            "ACCEPT",
        ])?;
        self.run_iptables(&[
            "-A",
            "FORWARD",
            "-i",
            &self.wan_iface,
            "-o",
            &self.zt_iface,
            "-m",
            "state",
            "--state",
            "ESTABLISHED,RELATED",
            "-j",
            "ACCEPT",
        ])
    }

    fn remove_iptables(&self) -> Result<(), RulesError> {
        // -D удаляет правило (игнорируем ошибки — правило могло уже не существовать)
        let _ = self.run_iptables(&[
            "-t",
            "nat",
            "-D",
            "POSTROUTING",
            "-i",
            &self.zt_iface,
            "-o",
            &self.wan_iface,
            "-j",
            "MASQUERADE",
        ]);
        let _ = self.run_iptables(&[
            "-D",
            "FORWARD",
            "-i",
            &self.zt_iface,
            "-o",
            &self.wan_iface,
            "-j",
            "ACCEPT",
        ]);
        let _ = self.run_iptables(&[
            "-D",
            "FORWARD",
            "-i",
            &self.wan_iface,
            "-o",
            &self.zt_iface,
            "-m",
            "state",
            "--state",
            "ESTABLISHED,RELATED",
            "-j",
            "ACCEPT",
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
}
