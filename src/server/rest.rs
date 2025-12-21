use std::sync::Arc;

use anyhow::Result;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Serialize;

use super::Server;
use crate::client::command::ClientCommand;
use crate::client::info::ClientInfo;

pub trait RestServer {
    async fn serve_rest(self: Arc<Self>) -> Result<()>;
}

impl RestServer for Server {
    async fn serve_rest(self: Arc<Self>) -> Result<()> {
        let rest_address = &self.settings.rest_address;
        let listener = tokio::net::TcpListener::bind(rest_address).await.unwrap();
        axum::serve(listener, router(self.clone())).await?;
        Ok(())
    }
}

fn router(server: Arc<Server>) -> Router {
    Router::new()
        .route("/v1/clients/online", get(list_online_clients))
        .route("/v1/clients/{imei}/log", get(get_client_log))
        .route("/v1/clients/command", post(send_command))
        .with_state(server)
}

async fn list_online_clients(State(server): State<Arc<Server>>) -> Json<Vec<ClientInfo>> {
    let clients = server.list_online_clients_impl().await;
    Json(clients)
}

async fn get_client_log(State(server): State<Arc<Server>>, Path(imei): Path<String>) -> String {
    match server.get_client_log_impl(&imei).await {
        Some(content) => content,
        None => String::from(""),
    }
}

#[derive(Serialize)]
struct CommandResponse {
    success: bool,
}

async fn send_command(
    State(server): State<Arc<Server>>,
    Json(command): Json<ClientCommand>,
) -> Json<CommandResponse> {
    let success = server.send_command_impl(&command);
    Json(CommandResponse { success })
}
