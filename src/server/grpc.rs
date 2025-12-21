use std::sync::Arc;

use anyhow::Result;
use tonic::{Request, Response, Status};

use self::grpc::controller_server::Controller;
use self::grpc::*;
use super::Server;
use crate::client::command::ClientCommand;

pub use self::grpc::ClientInfo as ProtoClientInfo;
pub use self::grpc::SendCommandToClientRequest;
use self::grpc::controller_server::ControllerServer;

mod grpc {
    tonic::include_proto!("controller");
}

pub trait GrpcServer {
    async fn serve_grpc(self: Arc<Self>) -> Result<()>;
}

impl GrpcServer for Server {
    async fn serve_grpc(self: Arc<Self>) -> Result<()> {
        let grpc_address = self.settings.grpc_address.clone().parse()?;

        tonic::transport::Server::builder()
            .add_service(ControllerServer::new(self))
            .serve(grpc_address)
            .await?;
        Ok(())
    }
}

#[tonic::async_trait]
impl Controller for Arc<Server> {
    #[doc = "Get the list of online clients"]
    async fn get_online_clients(
        &self,
        _request: Request<OnlineClientsRequest>,
    ) -> Result<Response<OnlineClientsResponse>, Status> {
        let proto_clients: Vec<ProtoClientInfo> = self
            .list_online_clients_impl()
            .await
            .into_iter()
            .map(|c| c.into())
            .collect();

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
        let log_content = self.get_client_log_impl(imei).await;

        let response = ClientLogResponse { log: log_content };

        Ok(Response::new(response))
    }

    #[doc = "Send command to specific client, imei for null means broadcast to all clients"]
    async fn send_command(
        &self,
        request: Request<SendCommandToClientRequest>,
    ) -> Result<Response<SendCommandToClientResponse>, Status> {
        let command: ClientCommand = request.into_inner().into();
        let success = self.send_command_impl(&command);

        let response = SendCommandToClientResponse { success };
        Ok(Response::new(response))
    }
}
