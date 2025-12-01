use log::{error, info};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::broadcast;

mod server;
mod settings;

#[tokio::main]
async fn main() {
    env_logger::init();

    let settings = settings::load_from_file("settings.json").await.unwrap();
    let address = format!("{}:{}", settings.bind_ip, settings.bind_port);

    let (msg_tx, _) = broadcast::channel::<String>(16);

    info!("starting server at {}", address);
    tokio::spawn(server::server_loop(address, msg_tx.clone()));

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
