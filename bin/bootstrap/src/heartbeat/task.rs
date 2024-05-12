use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use protocol::core::frames::ProtocolMessage;
use crate::types::AppStateRc;

async fn ping(
    server: &SocketAddr,
) -> Result<(), String> {
    let message = ProtocolMessage::Ping.into_frames().map_err(|e| format!("Failed to convert message into frames: {e}"))?;

    let mut stream = TcpStream::connect(server).await.map_err(|e| format!("Failed to connect to listed server: {e}"))?;

    for chunk in message {
        stream.write(&chunk).await.map_err(|e| format!("Failed to write to stream: {e}"))?;
    }

    Ok(())
}

pub async fn heartbeat_task(
    app_state: AppStateRc,
) {
    let servers = app_state
        .read()
        .expect("Failed to get read lock")
        .servers
        .keys()
        .cloned() // clone, not to keep the lock while making a bunch of requests
        .collect::<Vec<_>>();

    let mut dead_servers = vec![];
    for server in servers {
        if let Err(e) = ping(&server).await {
            println!("[INFO] Server {server} is dead - {e}");
            dead_servers.push(server);
        }
    }

    {
        let mut lock = app_state
            .write()
            .expect("Failed to get write lock");
        for server in dead_servers {
            lock.servers.remove(&server);
        }
    }
}
