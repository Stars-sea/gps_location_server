use log::info;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::broadcast,
};

async fn client_handler(
    mut client: TcpStream,
    client_addr: std::net::SocketAddr,
    mut msg_rx: broadcast::Receiver<String>,
) {
    info!("client connected: {}", client_addr);

    let mut client_data = vec![0u8; 1024];
    loop {
        tokio::select! {
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
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        info!("client {} lagged", client_addr);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("broadcast channel closed");
                        break;
                    }
                }
            }

            read_result = client.read(&mut client_data) => {
                match read_result {
                    Ok(0) => {
                        info!("client {} disconnected", client_addr);
                        break;
                    }
                    Ok(n) => {
                        let received = String::from_utf8_lossy(&client_data[..n]);
                        info!("received from {}: {}", client_addr, received);
                    }
                    Err(e) => {
                        log::error!("failed to read from client {}: {}", client_addr, e);
                        break;
                    }
                }
            }
        }
    }
    info!("client disconnected: {}", client_addr);
}

pub async fn server_loop(addr: String, msg_tx: broadcast::Sender<String>) {
    let listener = TcpListener::bind(addr).await.unwrap();

    loop {
        let (client, client_sock_addr) = listener.accept().await.unwrap();
        let msg_rx = msg_tx.subscribe();
        tokio::spawn(client_handler(client, client_sock_addr, msg_rx));
    }
}
