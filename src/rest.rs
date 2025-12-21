use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use serde::Serialize;

use crate::client::command::ClientCommand;
use crate::client::info::ClientInfo;
use crate::server::Server;

pub fn router(server: Arc<Server>) -> Router {
    Router::new()
        .route("/v1/clients/online", get(list_online_clients))
        .route("/v1/clients/:imei/log", get(get_client_log))
        .route("/v1/clients/command", post(send_command))
        .with_state(server)
}

async fn list_online_clients(State(server): State<Arc<Server>>) -> Json<Vec<ClientInfo>> {
    let clients = server.list_online_clients_impl().await;
    Json(clients)
}

#[derive(Serialize)]
struct LogResponse {
    log: Option<String>,
}

async fn get_client_log(
    State(server): State<Arc<Server>>,
    Path(imei): Path<String>,
) -> Json<LogResponse> {
    let log = server.get_client_log_impl(&imei).await;
    Json(LogResponse { log })
}

#[derive(Serialize)]
struct CommandResponse {
    success: bool,
}

async fn send_command(
    State(server): State<Arc<Server>>,
    Json(command): Json<ClientCommand>,
) -> Json<CommandResponse> {
    let success = server.send_command_impl(command);
    Json(CommandResponse { success })
}
