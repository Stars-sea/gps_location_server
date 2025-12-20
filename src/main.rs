use anyhow::Result;
use log::{error, info};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::broadcast;
use tonic::transport::Server;

mod client_handler;
mod client_info;
pub mod server;
mod settings;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let settings = settings::load_from_file("settings.json").await?;
    let address = settings.address.clone();
    let grpc_address = settings.grpc_address.clone().parse()?;

    let (msg_tx, _) = broadcast::channel::<String>(16);

    let server = server::Server::new(settings, msg_tx.clone());
    let server_clone = server.clone();

    // Start TCP server loop
    info!("starting server at {}", address);
    tokio::spawn(async move { server_clone.server_loop().await.unwrap() });

    // Start console input loop
    info!("starting console input loop");
    tokio::spawn(async move { console_loop(msg_tx).await });

    // Start gRPC server
    info!("starting gRPC server at {}", grpc_address);
    Server::builder()
        .add_service(server::ControllerServer::new(server))
        .serve(grpc_address)
        .await?;
    Ok(())
}

async fn console_loop(msg_tx: broadcast::Sender<String>) {
    let mut stdin = BufReader::new(tokio::io::stdin());
    let mut line = String::new();

    loop {
        line.clear();
        match stdin.read_line(&mut line).await {
            Ok(0) => {
                info!("stdin closed.");
                break;
            }
            Ok(_) => {
                if msg_tx.send(line.trim().to_string()).is_err() {
                    info!("no active receivers");
                }
            }
            Err(e) => {
                error!("failed to read from stdin: {}", e);
                break;
            }
        }
    }
}
