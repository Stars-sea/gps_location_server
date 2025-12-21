use std::fmt::Display;
use std::net::SocketAddr;
use std::{path::PathBuf, time::Duration};

use anyhow::{Result, anyhow};
use chrono::Utc;
use log::{debug, error, info, warn};
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::broadcast::{self, error::RecvError};

use super::command::ClientCommand;
use super::info::ClientInfo;

pub struct ClientHandler {
    client: TcpStream,
    client_addr: SocketAddr,
    command_rx: broadcast::Receiver<ClientCommand>,
    heartbeat_duration: Duration,
    output_dir: String,

    client_info: Option<ClientInfo>,
    output_writer: Option<File>,
}

impl ClientHandler {
    pub fn new(
        client: TcpStream,
        client_addr: SocketAddr,
        command_rx: broadcast::Receiver<ClientCommand>,
        heartbeat_duration: Duration,
        output_dir: String,
    ) -> Self {
        Self {
            client,
            client_addr,
            command_rx,
            heartbeat_duration,
            output_dir,
            client_info: None,
            output_writer: None,
        }
    }

    pub async fn verify_client(&mut self) -> Result<ClientInfo> {
        let mut client_data = vec![0u8; 1024];

        let read_result = self.client.read(&mut client_data).await;
        self.handle_read_result(read_result, &mut client_data)
            .await?;

        return self
            .client_info
            .clone()
            .ok_or(anyhow!("failed to verify client"));
    }

    pub async fn run(&mut self) {
        info!(target: "client_handler", "{} connected", self);

        let mut client_data = vec![0u8; 1024];

        loop {
            tokio::select! {
                biased;

                read_result = self.client.read(&mut client_data) => {
                    if self.handle_read_result(read_result, &mut client_data).await.is_err() {
                        break;
                    }
                }

                console_data = self.command_rx.recv() => {
                    if self.handle_client_command(console_data).await.is_err() {
                        break;
                    }
                }

                _ = tokio::time::sleep(self.heartbeat_duration), if self.heartbeat_duration.as_secs() > 0 => {
                    warn!(target: "client_handler", "{} timed out due to inactivity", self);
                    break;
                }
            }
        }

        info!(target: "client_handler", "{} disconnected", self);
        self.shutdown_client().await;
    }

    pub fn identifier(&self) -> Option<String> {
        self.client_info.as_ref().map(|info| info.identifier())
    }

    async fn handle_read_result(
        &mut self,
        read_result: tokio::io::Result<usize>,
        client_data: &mut [u8],
    ) -> Result<()> {
        if let Err(e) = read_result {
            error!(target: "client_handler", "failed to read from {}: {}", self, e);
            return Err(e.into());
        }

        let read_len = read_result?;
        if read_len == 0 {
            return Err(anyhow!("client disconnected"));
        }

        let received = String::from_utf8_lossy(&client_data[..read_len]);
        if received == "HEARTBEAT" {
            debug!(target: "client_handler", "received heartbeat from {}", self);
            return Ok(());
        }

        info!(target: "client_handler", "received from {}: {}", self, received);
        if let Err(e) = self.handle_received_data(&received).await {
            error!(target: "client_handler", "failed to handle data from {}: {}", self, e);
            return Err(e.into());
        }

        Ok(())
    }

    async fn handle_received_data(&mut self, data: &str) -> Result<()> {
        if self.client_info.is_none() {
            let info = ClientInfo::from_json(&data).unwrap();
            let id = info.identifier();

            self.client_info.replace(info);
            info!(target: "client_handler", "{self} registered");

            let path = log_path(&self.output_dir, &id);
            let file = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .await?;
            self.output_writer.replace(file);
            return Ok(());
        }

        let time = Utc::now().to_rfc3339();
        let log_entry = format!("{} {}\n", time, data);
        let writer = self.output_writer.as_mut().unwrap();

        writer.write_all(log_entry.as_bytes()).await?;
        writer.flush().await?;

        Ok(())
    }

    async fn handle_client_command(
        &mut self,
        console_result: Result<ClientCommand, RecvError>,
    ) -> Result<()> {
        if self.client_info.is_none() {
            return Ok(());
        }

        match console_result {
            Ok(command) => {
                if !command.is_targeted(&self.identifier().unwrap()) {
                    return Ok(());
                }

                let msg = format!("{}\n", command.command);
                if let Err(e) = self.client.write_all(msg.as_bytes()).await {
                    error!(target: "client_handler", "failed to write to {}: {}", self, e);
                    return Err(e.into());
                }
            }
            Err(RecvError::Lagged(_)) => {
                info!(target: "client_handler", "{} lagged", self);
            }
            Err(RecvError::Closed) => {
                error!(target: "client_handler", "broadcast channel closed");
                return Err(anyhow!("broadcast channel closed"));
            }
        }
        Ok(())
    }

    async fn shutdown_client(&mut self) {
        if let Some(writer) = self.output_writer.as_mut() {
            if let Err(e) = writer.shutdown().await {
                warn!(target: "client_handler", "failed to close output file for {}: {}", self, e);
            }

            self.output_writer = None;
        }

        self.client_info = None;
        self.client.shutdown().await.ok();
    }
}

impl Display for ClientHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.identifier() {
            Some(id) => write!(f, "client[addr={}, imei={}]", self.client_addr, id),
            None => write!(f, "client[addr={}]", self.client_addr),
        }
    }
}

pub fn log_path(output_dir: &str, id: &str) -> PathBuf {
    PathBuf::from(output_dir).join(id)
}
