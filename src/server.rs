use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use log::{debug, warn};
use tokio::fs;
use tokio::net::TcpListener;
use tokio::sync::{Mutex, broadcast};
use tokio::time;
use tonic::{Request, Response, Status};

use crate::client_command::ClientCommand;
use crate::client_handler::{self, ClientHandler};
use crate::client_info::ClientInfo;
use crate::server::grpc::controller_server::Controller;
use crate::server::grpc::*;
use crate::settings::Settings;

pub use crate::server::grpc::ClientInfo as ProtoClientInfo;
pub use crate::server::grpc::SendCommandToClientRequest;
pub use crate::server::grpc::controller_server::ControllerServer;

mod grpc {
    tonic::include_proto!("controller");
}

#[derive(Clone, Debug)]
pub struct Server {
    settings: Settings,
    command_tx: broadcast::Sender<ClientCommand>,
    online_clients: Arc<Mutex<Vec<ClientInfo>>>,
}

impl Server {
    pub fn new(settings: Settings, command_tx: broadcast::Sender<ClientCommand>) -> Self {
        Self {
            settings,
            command_tx,
            online_clients: Arc::new(Mutex::new(Vec::new())),
        }
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
                online_clients.lock().await.push(info.clone());

                client_handler.run().await;

                // Remove client from online clients list on disconnect
                online_clients.lock().await.retain(|c| c != &info);
            });
        }
    }
}

#[tonic::async_trait]
impl Controller for Server {
    #[doc = "Get the list of online clients"]
    async fn get_online_clients(
        &self,
        _request: Request<OnlineClientsRequest>,
    ) -> Result<Response<OnlineClientsResponse>, Status> {
        let proto_clients: Vec<ProtoClientInfo> = self
            .online_clients
            .lock()
            .await
            .iter()
            .cloned()
            .map(|c| c.into())
            .collect();

        debug!(target: "gRPC", "online clients: {:?}", proto_clients);

        let response = OnlineClientsResponse {
            clients: proto_clients,
        };

        Ok(Response::new(response))
    }

    #[doc = "Get the log of specific client"]
    async fn get_client_log(
        &self,
        request: Request<ClientLogRequest>,
    ) -> Result<Response<ClientLogResponse>, Status> {
        let imei = &request.get_ref().imei;

        let log_path = client_handler::log_path(&self.settings.output_dir, imei);
        let log_content = fs::read_to_string(&log_path).await.ok();

        debug!(target: "gRPC", "client log for {}: {:?}", imei, log_content);

        let response = ClientLogResponse { log: log_content };

        Ok(Response::new(response))
    }

    #[doc = "Send command to specific client, imei for null means broadcast to all clients"]
    async fn send_command(
        &self,
        request: Request<SendCommandToClientRequest>,
    ) -> Result<Response<SendCommandToClientResponse>, Status> {
        let command: ClientCommand = request.into_inner().into();

        let send_err = self.command_tx.send(command.clone()).err();
        if send_err.is_some() {
            warn!(target: "gRPC", "no active receivers for command: {:?}", command);
        }

        let response = SendCommandToClientResponse {
            success: send_err.is_none(),
        };
        Ok(Response::new(response))
    }
}
