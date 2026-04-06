use super::platform::FirewallBackend;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RulesError {
    #[error("Unsupported firewall backend")]
    Unsupported,
    #[error("Command failed: {0}")]
    Command(String),
}

pub type Result<T> = std::result::Result<T, RulesError>;

pub fn enable_exit_node(backend: &FirewallBackend, zt_iface: &str, wan_iface: &str) -> Result<()> {
    match backend {
        FirewallBackend::Unsupported => Err(RulesError::Unsupported),
        FirewallBackend::Nftables => enable_nftables(zt_iface, wan_iface),
        FirewallBackend::Iptables => enable_iptables(zt_iface, wan_iface),
    }
}

pub fn disable_exit_node(backend: &FirewallBackend, zt_iface: &str) -> Result<()> {
    match backend {
        FirewallBackend::Unsupported => Err(RulesError::Unsupported),
        FirewallBackend::Nftables => disable_nftables(zt_iface),
        FirewallBackend::Iptables => disable_iptables(zt_iface),
    }
}

fn run(cmd: &str, args: &[&str]) -> Result<()> {
    let status = std::process::Command::new(cmd)
        .args(args)
        .status()
        .map_err(|e| RulesError::Command(e.to_string()))?;
    if status.success() {
        Ok(())
    } else {
        Err(RulesError::Command(format!("{cmd} exited with {status}")))
    }
}

fn enable_nftables(zt_iface: &str, wan_iface: &str) -> Result<()> {
    let rule = format!(
        "add rule ip nat POSTROUTING oifname \"{wan_iface}\" iifname \"{zt_iface}\" masquerade"
    );
    run("nft", &[&rule])
}

fn disable_nftables(_zt_iface: &str) -> Result<()> {
    run("nft", &["flush chain ip nat POSTROUTING"])
}

fn enable_iptables(zt_iface: &str, wan_iface: &str) -> Result<()> {
    run(
        "iptables",
        &[
            "-t",
            "nat",
            "-A",
            "POSTROUTING",
            "-i",
            zt_iface,
            "-o",
            wan_iface,
            "-j",
            "MASQUERADE",
        ],
    )
}

fn disable_iptables(zt_iface: &str) -> Result<()> {
    run(
        "iptables",
        &[
            "-t",
            "nat",
            "-D",
            "POSTROUTING",
            "-i",
            zt_iface,
            "-j",
            "MASQUERADE",
        ],
    )
}
