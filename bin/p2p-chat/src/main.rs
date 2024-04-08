mod frontend;
mod types;
mod utils;

use std::env::args;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::mpsc::channel;
use crate::frontend::setup_frontend;
use crate::frontend::state::{AppState, AppStateInner};
use crate::utils::ui::UITerminal;

use protocol::core::{
    commands::command_processor,
    client::start_client,
    server::handle_connection::start_server,
    state::ProtocolState,
};
use protocol::types::package::{AlertPackage, AlertPackageLevel, AppPackage};

fn main() {
    let (server_addr, client_addr) = {
        let mut args = args().skip(1);

        let mut server_addr: Option<SocketAddr> = None;
        let mut client_addr: Option<SocketAddr> = None;
        while let Some(arg) = args.next() {
            match arg.as_ref() {
                "-s" => {
                    let addr = args.next().expect("Missing an address of a server to bind this node to");
                    let addr = SocketAddr::from_str(&addr).expect("Invalid address for a server");
                    server_addr = Some(addr)
                },
                "-c" => {
                    let addr = args.next().expect("Missing an address of a server to connect to");
                    let addr = SocketAddr::from_str(&addr).expect("Invalid address for a client");
                    client_addr = Some(addr)
                }
                _ => {
                    panic!("Unknown argument {}", arg);
                }
            }
        }

        (
            server_addr.expect("Server is required to run node"),
            client_addr,
        )
    };

    let (package_sender, package_receiver) = channel();
    let (command_sender, command_receiver) = channel();

    let app_state = {
        let seed = 1234567; // todo: get seed randomly
        let protocol_state = ProtocolState::new(
            server_addr,
            command_sender,
            package_sender,
            seed,
        );
        AppState::new(AppStateInner {
            protocol_state,
            ui: UITerminal::new(),
        })
    };
    let mut handles = vec![];

    app_state.new_package(AppPackage::Alert(AlertPackage {
        level: AlertPackageLevel::INFO,
        msg: "Init threads".to_string(),
    }));

    {
        let protocol_state = app_state.protocol_state.clone();
        if let Some(handle) = start_server(protocol_state, server_addr) {
            handles.push(handle);
        }
    }

    // this will be removed with ui commands like `/connect`
    if let Some(client_addr) = client_addr {
        let protocol_state = app_state.protocol_state.clone();
        let handle = start_client(protocol_state, client_addr, None);
        handles.extend(handle);
    }

    handles.extend(command_processor(
        app_state.protocol_state.clone(),
        command_receiver, // this is a bridge from application to protocol
    ));
    handles.extend(setup_frontend(
        app_state.clone(),
        package_receiver, // this is a bridge from protocol to application
    ));

    for handle in handles {
        handle.join()
            .map_err(|e| panic!("---Error joining the thread - {:?}", e))
            .expect("---Thread panic'd");
    }
}
