use chrono::Utc;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use log::{debug, info, warn};
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{self, error::RecvError};

use crate::client_info::ClientInfo;
use crate::settings::Settings;

async fn handle_received_data(
    data: &str,
    client_info: &mut Option<ClientInfo>,
    output_dir: &String,
    output_writer: &mut Option<File>,
) -> Result<()> {
    if client_info.is_none() {
        let info = ClientInfo::from_json(&data).unwrap();
        client_info.replace(info.clone());
        info!("registered client: {:?}", info.identifier());

        let path = PathBuf::from(output_dir).join(info.identifier());
        let file = fs::File::create(path).await?;
        output_writer.replace(file);
        return Ok(());
    }

    let time = Utc::now().to_rfc3339();
    let log_entry = format!("{} {}\n", time, data);
    let writer = output_writer.as_mut().unwrap();

    writer.write_all(log_entry.as_bytes()).await?;
    writer.flush().await?;

    Ok(())
}

async fn client_handler(
    mut client: TcpStream,
    client_addr: std::net::SocketAddr,
    mut msg_rx: broadcast::Receiver<String>,
    heartbeat_duration: Duration,
    output_dir: String,
) -> Result<()> {
    let mut client_info: Option<ClientInfo> = None;
    let mut output_writer: Option<File> = None;

    info!("client connected: {}", client_addr);

    let mut client_data = vec![0u8; 1024];

    loop {
        tokio::select! {
            biased;

            read_result = client.read(&mut client_data) => {
                match read_result {
                    Ok(0) => {
                        info!("client {} disconnected", client_addr);
                        break;
                    }
                    Ok(n) => {
                        let received = String::from_utf8_lossy(&client_data[..n]);
                        if received == "HEARTBEAT" {
                            debug!("received heartbeat from {}", client_addr);
                            continue;
                        }
                        info!("received from {}: {}", client_addr, received);
                        handle_received_data(&received, &mut client_info, &output_dir, &mut output_writer).await?;
                    }
                    Err(e) => {
                        log::error!("failed to read from client {}: {}", client_addr, e);
                        break;
                    }
                }
            }

            console_data = msg_rx.recv() => {
                match console_data {
                    Ok(msg) => {
                        if let Err(e) = client.write_all(msg.as_bytes()).await {
                            log::error!("failed to write to client {}: {}", client_addr, e);
                            break;
                        }
                        if let Err(e) = client.write_all(b"\n").await {
                            log::error!("failed to write to client {}: {}", client_addr, e);
                            break;
                        }
                    }
                    Err(RecvError::Lagged(_)) => {
                        info!("client {} lagged", client_addr);
                    }
                    Err(RecvError::Closed) => {
                        info!("broadcast channel closed");
                        break;
                    }
                }
            }

            _ = tokio::time::sleep(heartbeat_duration), if heartbeat_duration.as_secs() > 0 => {
                warn!("client {} timed out due to inactivity", client_addr);
                break;
            }
        }
    }

    info!("client disconnected: {}", client_addr);
    if let Some(writer) = output_writer.as_mut() {
        writer.flush().await?;
        writer.shutdown().await?;
    }

    Ok(())
}

pub async fn server_loop(settings: Settings, msg_tx: broadcast::Sender<String>) -> Result<()> {
    let listener = TcpListener::bind(settings.address).await.unwrap();

    if !fs::try_exists(&settings.output_dir).await.unwrap_or(false) {
        fs::create_dir_all(&settings.output_dir).await?;
    }

    let heartbeat_duration = Duration::from_secs(settings.heartbeat_sec);

    loop {
        let (client, client_addr) = listener.accept().await.unwrap();
        let msg_rx = msg_tx.subscribe();
        tokio::spawn(client_handler(
            client,
            client_addr,
            msg_rx,
            heartbeat_duration,
            settings.output_dir.clone(),
        ));
    }
}
