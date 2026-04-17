//! Thin wrapper around the system `ssh` binary for remote relay deploy.
//!
//! We intentionally use the OS `ssh` rather than pulling in a Rust SSH crate:
//! - Avoids heavy crypto dependencies.
//! - Reuses the user's existing `~/.ssh/known_hosts` and agent.
//! - Works on every platform that ships OpenSSH.
//!
//! **Key-based authentication only.** Password authentication via `sshpass`
//! has been removed: `sshpass` exposes the password via process arguments
//! (visible in `ps aux`), requires an optional system package, and is
//! inherently less secure than key-based auth. Users must configure a key.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SshError {
    #[error("ssh binary not found: {0}")]
    NotFound(String),
    #[error("ssh command failed (exit {code}): {stderr}")]
    Failed { code: i32, stderr: String },
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct SshClient {
    pub host: String,
    pub port: u16,
    pub user: String,
    /// Path to the private key file on the *local* machine.
    /// If `None`, SSH will use the default key from `~/.ssh/`.
    pub key_path: Option<String>,
}

impl SshClient {
    /// Run a single command on the remote host; return stdout.
    pub fn run(&self, cmd: &str) -> Result<String, SshError> {
        let ssh = which::which("ssh").map_err(|e| SshError::NotFound(e.to_string()))?;

        let mut args: Vec<String> = vec![
            "-p".into(),
            self.port.to_string(),
            // accept-new: auto-accept the host key on first connect only.
            // Unlike StrictHostKeyChecking=no, it WILL reject changed keys on
            // subsequent connects — protecting against MITM after first use.
            "-o".into(),
            "StrictHostKeyChecking=accept-new".into(),
            // BatchMode=yes: never prompt interactively (fail instead).
            // This ensures key auth is required; password prompts are suppressed.
            "-o".into(),
            "BatchMode=yes".into(),
            // Abort TCP handshake after 15 s so a firewalled host doesn't
            // block the deploy handler indefinitely.
            "-o".into(),
            "ConnectTimeout=15".into(),
            // Detect a silent mid-session disconnect: send a keepalive every
            // 10 s and give up after 3 missed replies (30 s total).
            "-o".into(),
            "ServerAliveInterval=10".into(),
            "-o".into(),
            "ServerAliveCountMax=3".into(),
        ];

        if let Some(ref key) = self.key_path {
            args.push("-i".into());
            args.push(key.clone());
        }

        args.push(format!("{}@{}", self.user, self.host));
        args.push(cmd.to_string());

        let output = std::process::Command::new(ssh).args(&args).output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            Err(SshError::Failed {
                code: output.status.code().unwrap_or(-1),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ssh_client_fields() {
        let c = SshClient {
            host: "1.2.3.4".into(),
            port: 22,
            user: "root".into(),
            key_path: Some("/tmp/key".into()),
        };
        assert_eq!(c.host, "1.2.3.4");
        assert_eq!(c.port, 22);
        assert_eq!(c.key_path.as_deref(), Some("/tmp/key"));
    }

    #[test]
    fn ssh_client_no_key() {
        let c = SshClient {
            host: "1.2.3.4".into(),
            port: 22,
            user: "root".into(),
            key_path: None,
        };
        assert!(c.key_path.is_none());
    }
}
