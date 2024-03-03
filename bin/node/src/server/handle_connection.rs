use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread::JoinHandle;
use crate::protocol::frames::ProtocolMessage;
use crate::protocol::start_pinging::start_pinging;
use crate::protocol::read_stream::protocol_read_stream;
use crate::protocol::node_info::NodeInfo;
use crate::types::package::{AlertPackage, AlertPackageLevel, AppPackage};
use crate::types::state::{AppState, MetaData};

fn handle_connection(
    app_state: AppState,
    mut stream: TcpStream,
) {
    let addr;
    let pinging_handle;

    let first_message = ProtocolMessage::from_stream(&mut stream)
        .expect("---Failed to read stream")
        .expect("---Should receive at least init message");

    addr = match first_message {
        ProtocolMessage::ConnInit { server_addr } => server_addr,
        _ => {
            unreachable!("Unexpected first message")
        }
    };

    {
        let mut lock = app_state.write_lock().expect("---Failed to get write lock");

        lock
            .package_sender
            .send(AppPackage::Alert(AlertPackage {
                level: AlertPackageLevel::INFO,
                msg: format!("New join from {}", addr),
            }))
            .expect("---Failed to send app package");

        // todo: it's hardcode, provide choice to the user to change rooms
        AppState::set_selected_room(&mut lock, Some(addr));

        let mut conn_metadata = MetaData {
            ping: 0,
            ping_started_at: None,
            topology_rad: 0_f32,
            knows_about: vec![],
        };

        {
            let another_conn = lock.streams.iter().find(|(k, _)| !k.eq(&&addr));

            if let Some((targ_addr, (_, targ_metadata))) = another_conn {
                lock
                    .package_sender
                    .send(AppPackage::Alert(AlertPackage {
                        level: AlertPackageLevel::DEBUG,
                        msg: format!("Sending info about another node {}", targ_addr),
                    }))
                    .expect("---Failed to send app package");

                ProtocolMessage::NodeInfo(
                    NodeInfo::new(targ_addr.clone(), targ_metadata.ping)
                )
                    .send_to_stream(&mut stream)
                    .expect("---Failed to send protocol message");

                conn_metadata.knows_about.push(targ_addr.clone());
            }
        }

        lock.streams.insert(addr, (
            stream.try_clone().expect("---Failed to clone tcp stream"),
            conn_metadata
        ));
    }

    {
        let app_state = app_state.clone();
        pinging_handle = std::thread::spawn(move || {
            start_pinging(app_state, addr)
        })
    }

    protocol_read_stream(
        app_state,
        addr,
        stream,
    );

    pinging_handle.join().expect("Should exit gracefully");
}

fn running_server(
    app_state: AppState,
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
                let lock = app_state.write_lock().expect("---Failed to get write lock");
                lock
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
    app_state: AppState,
    server_addr: SocketAddr,
) -> Option<JoinHandle<()>> {
    let server = {
        let mut lock = app_state.write_lock().expect("---Failed to get write lock");

        let server = TcpListener::bind(server_addr).expect("---Failed to assign udp socket");

        lock
            .package_sender
            .send(AppPackage::Alert(AlertPackage {
                level: AlertPackageLevel::INFO,
                msg: format!("Listening on {}", server_addr),
            }))
            .expect("---Failed to send app package");

        lock.server_addr = server_addr;

        server
    };

    Some(
        std::thread::spawn(|| {
            running_server(app_state, server)
        })
    )
}
