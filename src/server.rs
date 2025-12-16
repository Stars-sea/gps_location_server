use std::time::Duration;

use anyhow::Result;
use tokio::fs;
use tokio::net::TcpListener;
use tokio::sync::broadcast;

use crate::client::ClientHandler;
use crate::settings::Settings;

pub async fn server_loop(settings: Settings, msg_tx: broadcast::Sender<String>) -> Result<()> {
    let listener = TcpListener::bind(settings.address).await.unwrap();

    if !fs::try_exists(&settings.output_dir).await.unwrap_or(false) {
        fs::create_dir_all(&settings.output_dir).await?;
    }

    let heartbeat_duration = Duration::from_secs(settings.heartbeat_sec);

    loop {
        let (client, client_addr) = listener.accept().await.unwrap();
        let mut client_handler = ClientHandler::new(
            client,
            client_addr,
            msg_tx.subscribe(),
            heartbeat_duration,
            settings.output_dir.clone(),
        );
        tokio::spawn(async move { client_handler.run().await });
    }
}
