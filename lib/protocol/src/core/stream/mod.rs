use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::mpsc::Receiver;
use crate::core::frames::ProtocolMessage;
use crate::types::state::{ProtocolState, StreamRequest};

pub mod read_stream;
pub mod ping_stream;
mod types;

use types::StreamAction;
use crate::types::package::{AlertPackage, AlertPackageLevel, AppPackage};

pub async fn protocol_handle_stream(
    protocol_state: ProtocolState,
    addr: SocketAddr,
    mut stream: TcpStream, // should be cloned anyway bc otherwise `&mut` at `stream.read` will block whole application
    mut stream_request_sender: Receiver<StreamRequest>
) {
    ping_stream::ping_action(&protocol_state, addr).await; // we need to start pinging right away

    loop {
        let action = select! {
            request = stream_request_sender.recv() => {
                if request.is_none() {
                    protocol_state
                        .read()
                        .package_sender
                        .send(AppPackage::Alert(AlertPackage {
                            level: AlertPackageLevel::WARNING,
                            msg: format!("stream_request_sender is closed, disconnecting from stream {}", addr)
                        }))
                        .await
                        .expect("Failed to send package message");
                    StreamAction::Disconnect
                } else {
                    match request.unwrap() {
                        StreamRequest::Disconnect => StreamAction::Disconnect,
                        StreamRequest::Send(message) => StreamAction::Send(message),
                    }
                }
            }
            message = ProtocolMessage::from_stream(&mut stream) => {
                let message = message.expect("---Failed to read stream");
                read_stream::read_message(&protocol_state, addr, message).await
            }
            _ = tokio::time::sleep(Duration::from_secs(ping_stream::PING_INTERVAL)) => {
                ping_stream::ping_action(&protocol_state, addr).await
            }
        };

        match action {
            StreamAction::None => {},
            StreamAction::Disconnect => {
                stream.shutdown().await.expect("Failed to shutdown the stream");
                protocol_state.lock().await.streams.remove(&addr);
                break;
            },
            StreamAction::Send(message) => {
                ProtocolState::send_message(
                    &mut stream,
                    message
                )
                    .await
                    .expect("Failed to send protocol to stream");
            }
        }
    }
}
