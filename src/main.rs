use log::{error, info};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::broadcast;

mod client_handler;
mod client_info;
mod server;
mod settings;

#[tokio::main]
async fn main() {
    env_logger::init();

    let settings = settings::load_from_file("settings.json").await.unwrap();

    let (msg_tx, _) = broadcast::channel::<String>(16);

    info!("starting server at {}", settings.address);
    tokio::spawn(server::server_loop(settings, msg_tx.clone()));

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
