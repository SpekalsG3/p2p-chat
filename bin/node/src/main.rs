mod server;
mod client;

use std::env::args;
use std::net::SocketAddr;
use std::str::FromStr;
use crate::client::start_client;
use crate::server::start_server;

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

    println!("---Init threads");

    let mut handles = vec![];

    if let Some(server_addr) = server_addr {
        let server_handle = std::thread::spawn(move || {
            start_server(server_addr)
        });
        handles.push(server_handle);
    }
    if let Some(client_addr) = client_addr {
        let client_handle = std::thread::spawn(move || {
            start_client(client_addr)
        });
        handles.push(client_handle);
    }

    for handle in handles {
        handle.join()
            .map_err(|e| panic!("---Error joining the thread - {:?}", e))
            .expect("---Thread panic'd");
    }
}
