use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::select;
use protocol::core::frames::ProtocolMessage;
use crate::types::AppStateRc;

async fn ping(
    server: &SocketAddr,
) -> Result<(), String> {
    let message = ProtocolMessage::Ping
        .into_frames()
        .map_err(|e| format!("Failed to convert message into frames: {e}"))?;

    let mut stream = TcpStream::connect(server)
        .await
        .map_err(|e| format!("Failed to connect to listed server: {e}"))?;

    for chunk in message {
        stream
            .write(&chunk)
            .await
            .map_err(|e| format!("Failed to write to stream: {e}"))?;
    }

    let res = select! {
        result = ProtocolMessage::from_stream(&mut stream) => {
            let (response, _) = result
                .map_err(|e| format!("Failed to read message: {e}"))?
                .ok_or("Stream closed before received response")?;
            match response {
                ProtocolMessage::Pong(_) => Ok(()),
                _ => Err("Expected Pong on Ping message".to_string()),
            }
        },
        _ = tokio::time::sleep(Duration::from_secs(1 * 60)) => {
            Err("Failed to receive timeout in 1 minute".to_string())
        }
    };

    // todo: Should it also send `ConnClosed` frame? Node suppose to close conn itself...
    stream.shutdown().await.expect("Failed to shutdown stream");

    res
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
