use std::sync::Arc;

use anyhow::Result;
use axum::extract::{Path, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::Serialize;

use super::Server;
use crate::client::command::ClientCommand;
use crate::client::info::{ClientInfo, RegisteredClientInfo};

pub trait RestServer {
    async fn serve_rest(self: Arc<Self>) -> Result<()>;
}

impl RestServer for Server {
    async fn serve_rest(self: Arc<Self>) -> Result<()> {
        let rest_address = &self.settings.rest.address;
        let listener = tokio::net::TcpListener::bind(rest_address).await.unwrap();
        axum::serve(listener, router(self.clone())).await?;
        Ok(())
    }
}

fn router(server: Arc<Server>) -> Router {
    Router::new()
        .route("/v1/clients/online", get(list_online_clients))
        .route("/v1/clients/history", get(list_registered_clients))
        .route("/v1/clients/{imei}/info", get(get_client_info))
        .route("/v1/clients/{imei}/log", get(get_client_log))
        .route("/v1/clients/command", post(send_command))
        .route("/v1/clients/{imei}/name/{name}", post(set_registered_name))
        .route("/v1/clients/{imei}/tags/{tag}", post(add_tag))
        .route("/v1/clients/{imei}/tags/{tag}", delete(remove_tag))
        .with_state(server)
}

async fn list_online_clients(State(server): State<Arc<Server>>) -> Json<Vec<ClientInfo>> {
    let clients = server.list_online_clients_impl().await;
    Json(clients)
}

async fn list_registered_clients(
    State(_server): State<Arc<Server>>,
) -> Json<Vec<RegisteredClientInfo>> {
    let clients = RegisteredClientInfo::load().await.unwrap_or_default();
    Json(clients)
}

async fn get_client_info(
    State(_server): State<Arc<Server>>,
    Path(imei): Path<String>,
) -> Json<Option<RegisteredClientInfo>> {
    let info = RegisteredClientInfo::find(&imei).await;
    Json(info)
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

#[derive(Serialize)]
struct InfoOperationResponse {
    success: bool,
}

async fn set_registered_name(
    State(_server): State<Arc<Server>>,
    Path((imei, name)): Path<(String, String)>,
) -> Json<InfoOperationResponse> {
    let info = RegisteredClientInfo::find(&imei).await;
    if info.is_none() {
        return Json(InfoOperationResponse { success: false });
    }

    let mut info = info.unwrap();
    info.set_registered_name(name);
    let success = info.save().await.is_ok();
    Json(InfoOperationResponse { success })
}

async fn add_tag(
    State(_server): State<Arc<Server>>,
    Path((imei, tag)): Path<(String, String)>,
) -> Json<InfoOperationResponse> {
    let info = RegisteredClientInfo::find(&imei).await;
    if info.is_none() {
        return Json(InfoOperationResponse { success: false });
    }

    let mut info = info.unwrap();
    info.add_tag(tag);
    let success = info.save().await.is_ok();
    Json(InfoOperationResponse { success })
}

async fn remove_tag(
    State(_server): State<Arc<Server>>,
    Path((imei, tag)): Path<(String, String)>,
) -> Json<InfoOperationResponse> {
    let info = RegisteredClientInfo::find(&imei).await;
    if info.is_none() {
        return Json(InfoOperationResponse { success: false });
    }

    let mut info = info.unwrap();
    info.remove_tag(&tag);
    let success = info.save().await.is_ok();
    Json(InfoOperationResponse { success })
}
