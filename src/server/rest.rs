use std::sync::Arc;

use anyhow::Result;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::Server;
use crate::client::command::ClientCommand;
use crate::client::info::RegisteredClientInfo;

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
        .route("/v1/clients", get(list_all_clients))
        .route("/v1/clients/online", get(list_online_clients))
        .route("/v1/clients/{imei}/info", get(get_client_info))
        .route("/v1/clients/{imei}/log", get(get_client_log))
        .route("/v1/clients/command", post(send_command))
        .route("/v1/clients/{imei}/meta", post(set_meta))
        .with_state(server)
}

#[derive(Serialize, Debug)]
struct ClientInfoResponse {
    pub imei: String,
    pub iccid: String,
    pub fver: String,

    pub csq: Option<i32>,

    pub name: Option<String>,
    pub tags: Vec<String>,

    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

impl From<RegisteredClientInfo> for ClientInfoResponse {
    fn from(info: RegisteredClientInfo) -> Self {
        Self {
            imei: info.base_info.imei,
            iccid: info.base_info.iccid,
            fver: info.base_info.fver,
            csq: Some(info.base_info.csq),
            name: info.name,
            tags: info.tags,
            first_seen: info.first_seen,
            last_seen: info.last_seen,
        }
    }
}

async fn list_clients(server: Arc<Server>) -> Result<Vec<ClientInfoResponse>> {
    let online_clients = server.list_online_clients_impl().await;
    let clients = RegisteredClientInfo::load()
        .await?
        .into_iter()
        .map(|info| {
            let mut info: ClientInfoResponse = info.into();

            let online = online_clients.iter().find(|&c| c.imei == info.imei);
            info.csq = online.map(|c| c.csq);
            info
        })
        .collect();
    Ok(clients)
}

async fn list_all_clients(State(server): State<Arc<Server>>) -> Json<Vec<ClientInfoResponse>> {
    let clients = list_clients(server).await.unwrap_or_default();
    Json(clients)
}

async fn list_online_clients(State(server): State<Arc<Server>>) -> Json<Vec<ClientInfoResponse>> {
    let clients = list_clients(server).await.unwrap_or_default();
    let clients = clients.into_iter().filter(|c| c.csq.is_some()).collect();
    Json(clients)
}

async fn get_client_info(
    State(server): State<Arc<Server>>,
    Path(imei): Path<String>,
) -> Json<Option<ClientInfoResponse>> {
    let info = RegisteredClientInfo::find(&imei).await;
    if info.is_none() {
        return Json(None);
    }

    let mut info: ClientInfoResponse = info.unwrap().into();

    let online_clients = server.list_online_clients_impl().await;
    let online = online_clients.iter().find(|&c| c.imei == info.imei);
    info.csq = online.map(|c| c.csq);

    Json(Some(info))
}

async fn get_client_log(State(server): State<Arc<Server>>, Path(imei): Path<String>) -> String {
    match server.get_client_log_impl(&imei).await {
        Some(content) => content,
        None => String::new(),
    }
}

#[derive(Serialize)]
struct OperationResponse {
    success: bool,
}

async fn send_command(
    State(server): State<Arc<Server>>,
    Json(command): Json<ClientCommand>,
) -> Json<OperationResponse> {
    let success = server.send_command_impl(&command);
    Json(OperationResponse { success })
}

#[derive(Deserialize)]
struct SetMetadataRequest {
    name: Option<String>,
    tags: Vec<String>,
}

async fn set_meta(
    State(_server): State<Arc<Server>>,
    Path(imei): Path<String>,
    Json(request): Json<SetMetadataRequest>,
) -> Json<OperationResponse> {
    let info = RegisteredClientInfo::find(&imei).await;
    if info.is_none() {
        return Json(OperationResponse { success: false });
    }

    let mut info = info.unwrap();
    if let Some(name) = request.name {
        info.set_name(name);
    }
    info.tags = request.tags;

    let success = info.save().await.is_ok();
    Json(OperationResponse { success })
}
