use std::sync::Arc;

use anyhow::Result;
use log::{error, info};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::broadcast;
use tonic::transport::Server;

mod client {
    pub mod command;
    pub mod handler;
    pub mod info;
}
mod rest;
pub mod server;
mod settings;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let settings = settings::load_from_file("settings.json").await?;
    let address = settings.address.clone();
    let grpc_address = settings.grpc_address.clone().parse()?;
    let rest_address = settings.rest_address.clone();

    let (command_tx, _) = broadcast::channel::<client::command::ClientCommand>(16);

    let server = Arc::new(server::Server::new(settings, command_tx.clone()));

    // Start TCP server loop
    let server_clone = server.clone();
    info!(target: "main", "starting server at {}", address);
    tokio::spawn(async move { server_clone.server_loop().await.expect("server loop error") });

    // Start console input loop
    info!(target: "main", "starting console input loop");
    tokio::spawn(async move { console_loop(command_tx).await.expect("console loop error") });

    // Start REST server
    let rest_server = server.clone();
    info!(target: "main", "starting REST server at {}", rest_address);
    tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(rest_address).await.unwrap();
        axum::serve(listener, rest::router(rest_server))
            .await
            .unwrap();
    });

    // Start gRPC server
    info!(target: "main", "starting gRPC server at {}", grpc_address);
    Server::builder()
        .add_service(server::ControllerServer::new(server))
        .serve(grpc_address)
        .await?;
    Ok(())
}

async fn console_loop(command_tx: broadcast::Sender<client::command::ClientCommand>) -> Result<()> {
    let mut stdin = BufReader::new(tokio::io::stdin());
    let mut line = String::new();

    loop {
        line.clear();
        let size = stdin.read_line(&mut line).await;
        if let Err(e) = size {
            error!(target: "console", "failed to read from stdin: {}", e);
            return Err(e.into());
        }

        let size = size?;
        if size == 0 {
            info!(target: "console", "stdin closed.");
            return Ok(());
        }

        let command = client::command::parse_command(line.trim());
        if command_tx.send(command).is_err() {
            info!(target: "console", "no active receivers");
        }
    }
}
