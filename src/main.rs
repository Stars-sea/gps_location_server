use std::sync::Arc;

use anyhow::Result;
use log::{error, info};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::broadcast;

#[cfg(feature = "grpc")]
use crate::server::grpc::GrpcServer;
#[cfg(feature = "rest")]
use crate::server::rest::RestServer;

mod client {
    pub mod command;
    pub mod handler;
    pub mod info;
}
mod server;
mod settings;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    println!(
        "Starting {} (version {})...",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    println!("Powered by {}", env!("CARGO_PKG_AUTHORS"));
    println!("Repository: {}\n", env!("CARGO_PKG_REPOSITORY"));

    info!(target: "main", "loading settings from settings.json");
    let settings = settings::load_from_file("settings.json").await?;

    let (command_tx, _) = broadcast::channel::<client::command::ClientCommand>(16);

    let server = Arc::new(server::Server::new(settings.clone(), command_tx.clone()));

    // Start TCP server loop
    let tcp_server = server.clone();
    info!(target: "main", "starting TCP server at {}", settings.address);
    tokio::spawn(async move { tcp_server.server_loop().await.expect("server loop error") });

    // Start REST server
    #[cfg(feature = "rest")]
    if settings.rest.enabled {
        let rest_server = server.clone();
        info!(target: "main", "starting REST server at {}", settings.rest.address);
        tokio::spawn(async move { rest_server.serve_rest().await.expect("REST server error") });
    }

    // Start gRPC server
    #[cfg(feature = "grpc")]
    if settings.grpc.enabled {
        info!(target: "main", "starting gRPC server at {}", settings.grpc.address);
        tokio::spawn(async move { server.serve_grpc().await.expect("gRPC server error") });
    }

    // Start console input loop
    info!(target: "main", "starting console input loop");
    console_loop(command_tx).await.expect("console loop error");

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

        let command = line.trim().parse()?;
        if command_tx.send(command).is_err() {
            info!(target: "console", "no active receivers");
        }
    }
}
