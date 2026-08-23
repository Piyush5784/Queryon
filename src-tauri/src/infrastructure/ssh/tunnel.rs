use std::sync::Arc;

use russh::client::{self, AuthResult};
use russh::keys::PrivateKeyWithHashAlg;
use tokio::net::TcpListener;

use crate::domain::connection::{SshAuth, SshTunnelConfig};
use crate::error::AppError;

pub struct SshTunnel {
    pub local_addr: std::net::SocketAddr,
    shutdown: tokio::sync::oneshot::Sender<()>,
}

impl SshTunnel {
    pub fn shutdown(self) {
        let _ = self.shutdown.send(());
    }
}

struct AcceptAllHostKeys;

impl client::Handler for AcceptAllHostKeys {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &russh::keys::PublicKeyOrCertificate,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

pub async fn open_tunnel(
    config: &SshTunnelConfig,
    target_host: &str,
    target_port: u16,
) -> Result<SshTunnel, AppError> {
    let ssh_config = Arc::new(client::Config::default());
    let mut handle = client::connect(ssh_config, (config.host.as_str(), config.port), AcceptAllHostKeys)
        .await
        .map_err(|e| AppError::new(format!("Could not reach SSH host {}:{}: {e}", config.host, config.port)))?;

    let auth_result = match &config.auth {
        SshAuth::Password { password } => handle
            .authenticate_password(&config.username, password)
            .await
            .map_err(|e| AppError::new(format!("SSH authentication failed: {e}")))?,
        SshAuth::PrivateKey { key_path, passphrase } => {
            let passphrase = if passphrase.is_empty() { None } else { Some(passphrase.as_str()) };
            let key = russh::keys::load_secret_key(key_path, passphrase).map_err(|e| {
                AppError::new(format!("Could not load SSH private key {key_path}: {e}"))
            })?;
            let key = PrivateKeyWithHashAlg::new(Arc::new(key), None);
            handle
                .authenticate_publickey(&config.username, key)
                .await
                .map_err(|e| AppError::new(format!("SSH authentication failed: {e}")))?
        }
    };

    match auth_result {
        AuthResult::Success => {}
        AuthResult::Failure { .. } => {
            return Err(AppError::new(
                "SSH authentication was rejected — check the username, password/key, and bastion access.",
            ));
        }
    }

    let listener = TcpListener::bind(("127.0.0.1", 0))
        .await
        .map_err(|e| AppError::new(format!("Could not open a local port for the SSH tunnel: {e}")))?;
    let local_addr = listener
        .local_addr()
        .map_err(|e| AppError::new(format!("Could not read the local tunnel port: {e}")))?;

    let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let handle = Arc::new(handle);
    let target_host = target_host.to_string();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut shutdown_rx => break,
                accepted = listener.accept() => {
                    let Ok((local_stream, peer_addr)) = accepted else { continue };
                    let handle = handle.clone();
                    let target_host = target_host.clone();
                    tokio::spawn(async move {
                        forward_one_connection(&handle, local_stream, peer_addr, &target_host, target_port).await;
                    });
                }
            }
        }
    });

    Ok(SshTunnel { local_addr, shutdown: shutdown_tx })
}

async fn forward_one_connection<H: client::Handler>(
    handle: &client::Handle<H>,
    mut local_stream: tokio::net::TcpStream,
    peer_addr: std::net::SocketAddr,
    target_host: &str,
    target_port: u16,
) {
    let channel = match handle
        .channel_open_direct_tcpip(target_host, target_port as u32, peer_addr.ip().to_string(), peer_addr.port() as u32)
        .await
    {
        Ok(c) => c,
        Err(e) => {
            log::warn!("SSH tunnel: failed to open channel to {target_host}:{target_port}: {e}");
            return;
        }
    };

    let mut remote_stream = channel.into_stream();
    if let Err(e) = tokio::io::copy_bidirectional(&mut local_stream, &mut remote_stream).await {
        log::debug!("SSH tunnel: connection to {target_host}:{target_port} ended: {e}");
    }
}
