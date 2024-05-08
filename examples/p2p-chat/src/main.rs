mod frontend;
mod types;
mod utils;

use std::env::args;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::sync::mpsc::channel;
use crate::frontend::setup_frontend;
use crate::frontend::state::{AppState, AppStateInner};
use crate::utils::ui::UITerminal;

use protocol::types::{
    builder::ProtocolBuilder,
    package::{AlertPackage, AlertPackageLevel, AppPackage},
};

#[tokio::main]
async fn main() {
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


    let (package_sender, package_receiver) = channel(10);
    let mut protocol_builder = ProtocolBuilder::new(
        server_addr,
        package_sender,
        1234567, // todo: get seed randomly
    );
    // this will be removed with ui commands like `/connect`
    if let Some(client_addr) = client_addr {
        protocol_builder.set_client(client_addr).await
    }
    let (protocol_state, protocol_handles) = protocol_builder.build().await;

    let app_state = AppState::new(AppStateInner {
        protocol_state,
        ui: UITerminal::new(),
    });
    let mut handles = vec![];

    app_state.new_package(AppPackage::Alert(AlertPackage {
        level: AlertPackageLevel::INFO,
        msg: "Init threads".to_string(),
    }));

    handles.extend(protocol_handles);

    handles.push(tokio::spawn(setup_frontend(
        app_state.clone(),
        package_receiver, // this is a bridge from protocol to application
    )));
    for join in handles {
        join
            .await
            .expect("---Thread panic'd");
    }
}
