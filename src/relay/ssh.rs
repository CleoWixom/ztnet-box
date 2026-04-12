//! Thin wrapper around the system `ssh` binary for remote relay deploy.
//!
//! We intentionally use the OS `ssh` rather than pulling in a Rust SSH crate:
//! - Avoids heavy crypto dependencies.
//! - Reuses the user's existing `~/.ssh/known_hosts` and agent.
//! - Works on every platform that ships OpenSSH.

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
    pub password: Option<String>,
    pub key_path: Option<String>,
}

impl SshClient {
    /// Run a single command on the remote host; return stdout.
    pub fn run(&self, cmd: &str) -> Result<String, SshError> {
        let ssh = which::which("ssh")
            .map_err(|e| SshError::NotFound(e.to_string()))?;

        let mut args: Vec<String> = vec![
            "-p".into(),
            self.port.to_string(),
            // Disable host key checking for automated deploys.
            // In production the user should pre-approve the host key.
            "-o".into(),
            "StrictHostKeyChecking=no".into(),
            "-o".into(),
            "BatchMode=yes".into(),
        ];

        if let Some(ref key) = self.key_path {
            args.push("-i".into());
            args.push(key.clone());
        }

        args.push(format!("{}@{}", self.user, self.host));
        args.push(cmd.to_string());

        let mut command = std::process::Command::new(ssh);
        command.args(&args);

        // Inject password via sshpass if provided
        let output = if let Some(ref pass) = self.password {
            let sshpass = which::which("sshpass")
                .map_err(|e| SshError::NotFound(format!("sshpass: {e}")))?;
            std::process::Command::new(sshpass)
                .arg("-p")
                .arg(pass)
                .arg("ssh")
                .args(&args[1..]) // skip redundant "ssh"
                .output()?
        } else {
            command.output()?
        };

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
            password: None,
            key_path: Some("/tmp/key".into()),
        };
        assert_eq!(c.host, "1.2.3.4");
        assert_eq!(c.port, 22);
        assert!(c.password.is_none());
    }
}
