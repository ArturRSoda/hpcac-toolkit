use anyhow::Result;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::process::Command;
use tokio::time::{Duration, Instant, sleep};

pub struct SshSession {
    pub ip: String,
    pub username: String,
    pub private_key_path: String,
    pub jump_host: Option<String>,
}

impl SshSession {
    fn display(&self) -> String {
        match &self.jump_host {
            Some(jump) => format!("{} (via {})", self.ip, jump),
            None => self.ip.clone(),
        }
    }

    pub fn for_aws(ip: &str, private_key_path: &str) -> Self {
        Self {
            ip: ip.to_string(),
            username: "ec2-user".to_string(),
            private_key_path: private_key_path.to_string(),
            jump_host: None,
        }
    }

    pub fn for_aws_worker(private_ip: &str, head_public_ip: &str, private_key_path: &str) -> Self {
        Self {
            ip: private_ip.to_string(),
            username: "ec2-user".to_string(),
            private_key_path: private_key_path.to_string(),
            jump_host: Some(format!("ec2-user@{}", head_public_ip)),
        }
    }

    fn base_args(&self) -> Vec<String> {
        let mut args = vec![
            "-i".to_string(),
            self.private_key_path.clone(),
            "-o".to_string(),
            "StrictHostKeyChecking=no".to_string(),
            "-o".to_string(),
            "UserKnownHostsFile=/dev/null".to_string(),
            "-o".to_string(),
            "ConnectTimeout=10".to_string(),
            "-o".to_string(),
            "ServerAliveInterval=30".to_string(),
            "-o".to_string(),
            "ServerAliveCountMax=3".to_string(),
            "-o".to_string(),
            "LogLevel=ERROR".to_string(),
        ];
        if let Some(ref jump) = self.jump_host {
            // ProxyCommand instead of ProxyJump so we can pass identity file to the jump hop too
            args.push("-o".to_string());
            args.push(format!(
                "ProxyCommand=ssh -i {} -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o LogLevel=ERROR -W %h:%p {}",
                self.private_key_path, jump
            ));
        }
        args.push(format!("{}@{}", self.username, self.ip));
        args
    }

    pub async fn wait_until_ready(&self, timeout: Duration) -> Result<()> {
        if self.jump_host.is_some() {
            return self.wait_until_ready_via_ssh(timeout).await;
        }
        let addr = format!("{}:22", self.ip);
        let deadline = Instant::now() + timeout;
        loop {
            if Instant::now() > deadline {
                anyhow::bail!(
                    "Timed out after {:?} waiting for SSH on '{}'",
                    timeout,
                    self.display()
                );
            }
            match tokio::time::timeout(Duration::from_secs(5), TcpStream::connect(&addr)).await {
                Ok(Ok(_)) => {
                    tracing::info!(
                        "SSH port open on '{}', waiting for sshd to initialize...",
                        self.display()
                    );
                    sleep(Duration::from_secs(5)).await;
                    return Ok(());
                }
                Ok(Err(_)) | Err(_) => {
                    tracing::info!("SSH not yet ready on '{}', retrying in 5s...", self.display());
                    sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    async fn wait_until_ready_via_ssh(&self, timeout: Duration) -> Result<()> {
        let deadline = Instant::now() + timeout;
        loop {
            if Instant::now() > deadline {
                anyhow::bail!(
                    "Timed out after {:?} waiting for SSH on '{}'",
                    timeout,
                    self.display()
                );
            }
            match self.run_command("echo ok").await {
                Ok(_) => {
                    tracing::info!("SSH ready on '{}'", self.display());
                    return Ok(());
                }
                Err(_) => {
                    tracing::info!("SSH not yet ready on '{}', retrying in 5s...", self.display());
                    sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    pub async fn run_command(&self, script: &str) -> Result<String> {
        tracing::info!("Running SSH command on '{}'", self.display());
        let mut args = self.base_args();
        args.push("bash -s".to_string());

        let mut child = Command::new("ssh")
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(script.as_bytes()).await?;
        }

        let output = child.wait_with_output().await?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            if !stdout.trim().is_empty() {
                tracing::error!("SSH stdout: {}", stdout);
            }
            anyhow::bail!("SSH command failed on '{}': {}", self.display(), stderr);
        }

        Ok(stdout)
    }
}
