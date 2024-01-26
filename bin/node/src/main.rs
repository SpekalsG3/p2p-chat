mod server;
mod client;
mod types;
mod frontend;
mod utils;

use std::env::args;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::mpsc::channel;
use crate::client::start_client;
use crate::frontend::handle_input::handle_input;
use crate::frontend::handle_packages::handle_packages;
use crate::frontend::setup_frontend;
use crate::server::handle_connection::start_server;
use crate::types::package::{AlertPackage, AlertPackageLevel, AppPackage};
use crate::types::state::AppState;

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

        (server_addr, client_addr)
    };

    let (package_sender, package_receiver) = channel();
    let app_state = AppState::new(package_sender);
    let mut handles = vec![];

    app_state
        .send_package(AppPackage::Alert(AlertPackage {
            level: AlertPackageLevel::INFO,
            msg: "Init threads".to_string(),
        }))
        .expect("---Failed to send package");

    if let Some(server_addr) = server_addr {
        let app_state = app_state.clone();
        let handle = std::thread::spawn(move || {
            start_server(app_state, server_addr)
        });
        handles.push(handle);
    }
    if let Some(client_addr) = client_addr {
        let app_state = app_state.clone();
        let handle = std::thread::spawn(move || {
            start_client(app_state, client_addr)
        });
        handles.push(handle);
    }
    handles.extend(setup_frontend(
        app_state.clone(),
        package_receiver,
    ));

    for handle in handles {
        handle.join()
            .map_err(|e| panic!("---Error joining the thread - {:?}", e))
            .expect("---Thread panic'd");
    }
}
