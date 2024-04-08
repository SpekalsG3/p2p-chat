use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread::JoinHandle;
use crate::core::{
    frames::ProtocolMessage,
    node_info::NodeInfo,
    start_pinging::start_pinging,
    read_stream::protocol_read_stream,
};
use crate::types::{
    state::{ProtocolState, StreamMetadata},
    package::{AlertPackage, AlertPackageLevel, AppPackage},
};

fn handle_connection(
    protocol_state: ProtocolState,
    mut stream: TcpStream,
) {
    let addr;
    let pinging_handle;

    {
        let (first_message, frames_count) = ProtocolMessage::from_stream(&mut stream)
            .expect("---Failed to read stream")
            .expect("---Should receive at least init message");

        addr = match first_message {
            ProtocolMessage::ConnInit { server_addr } => server_addr,
            _ => {
                unreachable!("Unexpected first message")
            }
        };

        let lock = &mut *protocol_state.lock().expect("---Failed to get write lock");

        for _ in 0..frames_count {
            lock.state.next();
        }

        protocol_state
            .read()
            .package_sender
            .send(AppPackage::Alert(AlertPackage {
                level: AlertPackageLevel::INFO,
                msg: format!("New join from {}", addr),
            }))
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
                    .expect("---Failed to send app package");

                conn_metadata.knows_about.push(targ_addr.clone());

                // todo: check angles and pings to find the closest node to the client
                //  idk the ping to this new connection nor who hes connected to
                //  i can think only of one thing - do the ping-pong first
                ProtocolState::send_message(
                    state,
                    &mut stream,
                    ProtocolMessage::NodeStatus(
                        NodeInfo::new(targ_addr.clone(), targ_metadata.ping)
                    ),
                )
                    .expect("---Failed to send protocol message");
            }
        }

        lock.streams.insert(addr, (
            stream.try_clone().expect("---Failed to clone tcp stream"),
            conn_metadata
        ));
    }

    {
        let app_state = protocol_state.clone();
        pinging_handle = std::thread::spawn(move || {
            start_pinging(app_state, addr)
        })
    }

    protocol_read_stream(
        protocol_state,
        addr,
        stream,
    );

    pinging_handle.join().expect("Should exit gracefully");
}

fn running_server(
    app_state: ProtocolState,
    server: TcpListener,
) {
    let mut handles = vec![];

    loop {
        match server.accept() {
            Ok((stream, _addr)) => { // I don't think I need an address of local socket
                let h = {
                    let app_state = app_state.clone();
                    std::thread::spawn(move || {
                        handle_connection(
                            app_state,
                            stream,
                        )
                    })
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
                    .expect("---Failed to send app package");
            }
        }
    }
}

pub fn start_server(
    protocol_state: ProtocolState,
    server_addr: SocketAddr,
) -> Option<JoinHandle<()>> {
    let server = {
        let server = TcpListener::bind(server_addr).expect("---Failed to assign udp socket");

        protocol_state
            .read()
            .package_sender
            .send(AppPackage::Alert(AlertPackage {
                level: AlertPackageLevel::INFO,
                msg: format!("Listening on {}", server_addr),
            }))
            .expect("---Failed to send app package");

        server
    };

    Some(
        std::thread::spawn(|| {
            running_server(protocol_state, server)
        })
    )
}
