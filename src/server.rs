use std::time::Duration;

use anyhow::Result;
use tokio::fs;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tonic::{Request, Response, Status};

use crate::client_handler::ClientHandler;
use crate::server::grpc::controller_server::Controller;
use crate::server::grpc::{
    ClientLogRequest, ClientLogResponse, OnlineClientsRequest, OnlineClientsResponse,
};
use crate::settings::Settings;

pub use crate::server::grpc::controller_server::ControllerServer;

mod grpc {
    tonic::include_proto!("controller");
}

#[derive(Clone, Debug)]
pub struct Server {
    settings: Settings,
    msg_tx: broadcast::Sender<String>,
}

impl Server {
    pub fn new(settings: Settings, msg_tx: broadcast::Sender<String>) -> Self {
        Self { settings, msg_tx }
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
            let mut client_handler = ClientHandler::new(
                client,
                client_addr,
                self.msg_tx.subscribe(),
                heartbeat_duration,
                self.settings.output_dir.clone(),
            );
            tokio::spawn(async move { client_handler.run().await });
        }
    }
}

#[tonic::async_trait]
impl Controller for Server {
    #[doc = "Get the list of online clients"]
    async fn get_online_clients(
        &self,
        request: Request<OnlineClientsRequest>,
    ) -> Result<Response<OnlineClientsResponse>, Status> {
        todo!()
    }

    #[doc = "Get the log of specific client"]
    async fn get_client_log(
        &self,
        request: Request<ClientLogRequest>,
    ) -> Result<Response<ClientLogResponse>, Status> {
        todo!()
    }
}
