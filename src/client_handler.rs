use std::net::SocketAddr;
use std::{path::PathBuf, time::Duration};

use anyhow::{Result, anyhow};
use chrono::Utc;
use log::{debug, error, info, warn};
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::broadcast::{self, error::RecvError};

use crate::client_info::ClientInfo;

pub struct ClientHandler {
    client: TcpStream,
    client_addr: SocketAddr,
    msg_rx: broadcast::Receiver<String>,
    heartbeat_duration: Duration,
    output_dir: String,

    client_info: Option<ClientInfo>,
    output_writer: Option<File>,
}

impl ClientHandler {
    pub fn new(
        client: TcpStream,
        client_addr: SocketAddr,
        msg_rx: broadcast::Receiver<String>,
        heartbeat_duration: Duration,
        output_dir: String,
    ) -> Self {
        Self {
            client,
            client_addr,
            msg_rx,
            heartbeat_duration,
            output_dir,
            client_info: None,
            output_writer: None,
        }
    }

    pub async fn run(&mut self) {
        info!("client connected: {}", self.client_addr);

        let mut client_data = vec![0u8; 1024];

        loop {
            tokio::select! {
                biased;

                read_result = self.client.read(&mut client_data) => {
                    if self.handle_read_result(read_result, &mut client_data).await.is_err() {
                        break;
                    }
                }

                console_data = self.msg_rx.recv() => {
                    if self.handle_console_data(console_data).await.is_err() {
                        break;
                    }
                }

                _ = tokio::time::sleep(self.heartbeat_duration), if self.heartbeat_duration.as_secs() > 0 => {
                    warn!("client {} timed out due to inactivity", self.client_addr);
                    break;
                }
            }
        }

        info!("client disconnected: {}", self.client_addr);
        self.shutdown_client().await;
    }

    async fn handle_read_result(
        &mut self,
        read_result: tokio::io::Result<usize>,
        client_data: &mut [u8],
    ) -> Result<()> {
        if let Err(e) = read_result {
            error!("failed to read from client {}: {}", self.client_addr, e);
            return Err(e.into());
        }

        let read_len = read_result?;
        if read_len == 0 {
            info!("client {} disconnected", self.client_addr);
            return Err(anyhow!("client disconnected"));
        }

        let received = String::from_utf8_lossy(&client_data[..read_len]);
        if received == "HEARTBEAT" {
            debug!("received heartbeat from {}", self.client_addr);
            return Ok(());
        }

        info!("received from {}: {}", self.client_addr, received);
        if let Err(e) = self.handle_received_data(&received).await {
            error!(
                "failed to handle data from client {}: {}",
                self.client_addr, e
            );
            return Err(e.into());
        }

        Ok(())
    }

    async fn handle_received_data(&mut self, data: &str) -> Result<()> {
        if self.client_info.is_none() {
            let info = ClientInfo::from_json(&data).unwrap();
            self.client_info.replace(info.clone());
            info!("registered client: {:?}", info.identifier());

            let path = PathBuf::from(&self.output_dir).join(info.identifier());
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

    async fn handle_console_data(
        &mut self,
        console_result: Result<String, RecvError>,
    ) -> Result<()> {
        match console_result {
            Ok(msg) => {
                let msg = format!("{msg}\n");
                if let Err(e) = self.client.write_all(msg.as_bytes()).await {
                    error!("failed to write to client {}: {}", self.client_addr, e);
                    return Err(e.into());
                }
            }
            Err(RecvError::Lagged(_)) => {
                info!("client {} lagged", self.client_addr);
            }
            Err(RecvError::Closed) => {
                info!("broadcast channel closed");
                return Err(anyhow!("broadcast channel closed"));
            }
        }
        Ok(())
    }

    async fn shutdown_client(&mut self) {
        if let Some(writer) = self.output_writer.as_mut() {
            if let Err(e) = writer.shutdown().await {
                warn!(
                    "failed to close output file for client {}: {}",
                    self.client_addr, e
                );
            }

            self.output_writer = None;
        }

        self.client_info = None;
        self.client.shutdown().await.ok();
    }
}
