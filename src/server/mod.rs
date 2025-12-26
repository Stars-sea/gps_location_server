use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use log::{debug, warn};
use tokio::fs;
use tokio::net::TcpListener;
use tokio::sync::{RwLock, broadcast};
use tokio::time;

use crate::client::command::ClientCommand;
use crate::client::handler::{self, ClientHandler};
use crate::client::info::ClientInfo;
use crate::settings::Settings;

#[cfg(feature = "rest")]
pub mod rest;

pub struct Server {
    settings: Settings,
    command_tx: broadcast::Sender<ClientCommand>,
    online_clients: Arc<RwLock<Vec<ClientInfo>>>,
}

impl Server {
    pub fn new(settings: Settings, command_tx: broadcast::Sender<ClientCommand>) -> Self {
        Self {
            settings,
            command_tx,
            online_clients: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn list_online_clients_impl(&self) -> Vec<ClientInfo> {
        debug!(target: "server", "listing online clients");
        self.online_clients.read().await.clone()
    }

    pub async fn get_client_log_impl(&self, imei: &str) -> Option<String> {
        debug!(target: "server", "getting client log for imei: {}", imei);
        let log_path = handler::log_path(&self.settings.output_dir, imei);
        fs::read_to_string(&log_path).await.ok()
    }

    pub fn send_command_impl(&self, command: &ClientCommand) -> bool {
        debug!(target: "server", "sending command: {}", command);

        let send_err = self.command_tx.send(command.clone()).err();
        if send_err.is_some() {
            warn!(target: "server", "no active receivers for command: {}", command);
        }
        send_err.is_none()
    }

    pub async fn server_loop(&self) -> Result<()> {
        let listener = TcpListener::bind(&self.settings.address).await.unwrap();

        let output_dir = &self.settings.output_dir;
        if !fs::try_exists(output_dir).await.unwrap_or(false) {
            fs::create_dir_all(output_dir).await?;
        }

        let heartbeat_duration = Duration::from_secs(self.settings.heartbeat_sec);

        loop {
            let (client, client_addr) = listener.accept().await.unwrap();
            let online_clients = self.online_clients.clone();
            let verify_timeout = Duration::from_secs(self.settings.verify_timeout);

            let mut client_handler = ClientHandler::new(
                client,
                client_addr,
                self.command_tx.subscribe(),
                heartbeat_duration,
                self.settings.output_dir.clone(),
            );
            tokio::spawn(async move {
                // Verify client and add to online clients list
                let info = time::timeout(verify_timeout, client_handler.verify_client())
                    .await
                    .unwrap()
                    .unwrap();
                online_clients.write().await.push(info.clone());

                client_handler.run().await;

                // Remove client from online clients list on disconnect
                online_clients.write().await.retain(|c| c != &info);
            });
        }
    }
}
