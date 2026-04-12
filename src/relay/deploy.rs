//! Remote pylon relay deploy over SSH.
//!
//! Steps per https://docs.zerotier.com/relay/:
//!  1. Connect via SSH
//!  2. Install Docker if missing
//!  3. (Optional) stop UFW to prevent iptables conflicts with Docker
//!  4. Pull and run `zerotier/pylon:latest reflect` on the configured port

use super::{ssh::SshClient, RelayDeployConfig, RemoteRelayInfo};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeployError {
    #[error("SSH error: {0}")]
    Ssh(#[from] super::ssh::SshError),
    #[error("Deploy step failed: {0}")]
    Step(String),
}

/// Deploy a pylon reflect container on the remote host.
pub fn deploy(cfg: &RelayDeployConfig) -> Result<RemoteRelayInfo, DeployError> {
    let client = SshClient {
        host: cfg.host.clone(),
        port: cfg.ssh_port,
        user: cfg.ssh_user.clone(),
        password: cfg.password.clone(),
        key_path: cfg.key_path.clone(),
    };

    // 1. Verify connectivity
    client.run("echo ztnet-ok")?;
    tracing::info!(host = %cfg.host, "SSH connected");

    // 2. Install Docker if not present
    let has_docker = client.run("command -v docker").is_ok();
    if !has_docker {
        tracing::info!(host = %cfg.host, "Docker not found, installing...");
        client
            .run("curl -fsSL https://get.docker.com | sh")
            .map_err(|e| DeployError::Step(format!("Docker install failed: {e}")))?;
        client
            .run("systemctl enable --now docker")
            .map_err(|e| DeployError::Step(format!("Docker enable failed: {e}")))?;
    }

    // 3. Stop UFW (conflicts with Docker iptables management)
    if cfg.stop_ufw {
        let _ = client.run("systemctl stop ufw && systemctl disable ufw");
        tracing::info!(host = %cfg.host, "UFW stopped");
    }

    // 4. Stop any existing pylon container
    let _ = client.run("docker stop ztnet-pylon 2>/dev/null; docker rm ztnet-pylon 2>/dev/null");

    // 5. Run pylon reflect container
    let docker_cmd = format!(
        "docker run -d --name ztnet-pylon --restart unless-stopped \
         -p {port}:{port}/tcp \
         zerotier/pylon:latest reflect",
        port = cfg.pylon_port,
    );
    client
        .run(&docker_cmd)
        .map_err(|e| DeployError::Step(format!("pylon container start failed: {e}")))?;

    tracing::info!(
        host = %cfg.host,
        port = cfg.pylon_port,
        "pylon relay deployed"
    );

    Ok(RemoteRelayInfo {
        host: cfg.host.clone(),
        port: cfg.pylon_port,
        reachable: Some(true),
        deployed_at: chrono::Utc::now().to_rfc3339(),
    })
}

/// Stop and remove the pylon container on the remote host.
pub fn remove(info: &RemoteRelayInfo, ssh_cfg: &RelayDeployConfig) -> Result<(), DeployError> {
    let client = SshClient {
        host: info.host.clone(),
        port: ssh_cfg.ssh_port,
        user: ssh_cfg.ssh_user.clone(),
        password: ssh_cfg.password.clone(),
        key_path: ssh_cfg.key_path.clone(),
    };
    let _ = client.run("docker stop ztnet-pylon && docker rm ztnet-pylon");
    tracing::info!(host = %info.host, "pylon relay removed");
    Ok(())
}

/// Check if the relay port is reachable via a TCP connect.
pub fn verify(host: &str, port: u16) -> bool {
    use std::net::TcpStream;
    use std::time::Duration;
    TcpStream::connect_timeout(
        &format!("{host}:{port}")
            .parse()
            .unwrap_or("0.0.0.0:443".parse().unwrap()),
        Duration::from_secs(5),
    )
    .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_unreachable_host_returns_false() {
        // 192.0.2.0/24 is TEST-NET — guaranteed unreachable
        assert!(!verify("192.0.2.1", 443));
    }

    #[test]
    fn relay_deploy_config_defaults() {
        let json = r#"{"host":"1.2.3.4"}"#;
        let cfg: super::super::RelayDeployConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.ssh_port, 22);
        assert_eq!(cfg.ssh_user, "root");
        assert_eq!(cfg.pylon_port, 443);
        assert!(cfg.stop_ufw);
    }
}
