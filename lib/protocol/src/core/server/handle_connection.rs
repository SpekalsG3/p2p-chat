use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;
use crate::core::{
    frames::ProtocolMessage,
    node_info::NodeInfo,
};
use crate::core::stream::protocol_handle_stream;
use crate::types::{
    state::{ProtocolState, StreamMetadata},
    package::{AlertPackage, AlertPackageLevel, AppPackage},
};

async fn handle_connection(
    protocol_state: ProtocolState,
    mut stream: TcpStream,
) {
    let addr;
    let stream_request_receiver;

    {
        let (first_message, _) = ProtocolMessage::from_stream(&mut stream)
            .await
            .expect("---Failed to read stream")
            .expect("---Should receive at least init message");

        addr = match first_message {
            ProtocolMessage::ConnInit { server_addr } => server_addr,
            _ => {
                unreachable!("Unexpected first message")
            }
        };

        let lock = &mut *protocol_state.lock().await;

        lock.state.next();

        protocol_state
            .read()
            .package_sender
            .send(AppPackage::Alert(AlertPackage {
                level: AlertPackageLevel::INFO,
                msg: format!("New join from {:?}", addr),
            }))
            .await
            .expect("---Failed to send app package");

        let mut conn_metadata = StreamMetadata::new();

        {
            let package_sender = &protocol_state.read().package_sender;
            let state = &mut lock.state;
            let streams = &mut lock.streams;

            let another_conn = streams.iter().find(|(k, _)| !k.eq(&&addr));

            if let Some((targ_addr, (_, targ_metadata))) = another_conn {
                package_sender
                    .send(AppPackage::Alert(AlertPackage {
                        level: AlertPackageLevel::DEBUG,
                        msg: format!("Sending info about another node {}", targ_addr),
                    }))
                    .await
                    .expect("---Failed to send app package");

                conn_metadata.knows_about.push(targ_addr.clone());

                // todo: check angles and pings to find the closest node to the client
                //  idk the ping to this new connection nor who hes connected to
                //  i can think only of one thing - do the ping-pong first
                state.next();
                ProtocolState::send_message(
                    &mut stream,
                    ProtocolMessage::NodeStatus(
                        NodeInfo::new(targ_addr.clone(), targ_metadata.ping)
                    ),
                )
                    .await
                    .expect("---Failed to send protocol message");
            }
        }

        let channels = tokio::sync::mpsc::channel(100);
        stream_request_receiver = channels.1;
        lock.streams.insert(addr.clone(), (channels.0, conn_metadata));
    }

    protocol_handle_stream(
        protocol_state,
        addr,
        stream,
        stream_request_receiver,
    ).await;
}

async fn running_server(
    app_state: ProtocolState,
    server: TcpListener,
) {
    let mut handles = vec![];

    loop {
        match server.accept().await {
            Ok((stream, _addr)) => { // I don't think I need an address of local socket
                let h = {
                    let app_state = app_state.clone();
                    tokio::spawn(handle_connection(
                        app_state,
                        stream,
                    ))
                };
                handles.push(h);
            },
            Err(e) => {
                app_state
                    .read()
                    .package_sender
                    .send(AppPackage::Alert(AlertPackage {
                        level: AlertPackageLevel::ERROR,
                        msg: format!("Failed to accept connection - {}", e),
                    }))
                    .await
                    .expect("---Failed to send app package");
            }
        }
    }

    for j in handles {
        j.await.expect("Thread failed");
    }
}

pub async fn start_server(
    protocol_state: ProtocolState,
    server_addr: SocketAddr,
) -> Option<JoinHandle<()>> {
    let server = {
        let server = TcpListener::bind(server_addr).await.expect("---Failed to assign udp socket");

        protocol_state
            .read()
            .package_sender
            .send(AppPackage::Alert(AlertPackage {
                level: AlertPackageLevel::INFO,
                msg: format!("Listening on {}", server_addr),
            }))
            .await
            .expect("---Failed to send app package");

        server
    };

    Some(
        tokio::spawn(
            running_server(protocol_state, server),
        )
    )
}
