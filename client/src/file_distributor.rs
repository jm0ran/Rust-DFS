/**
 * This will be our file distributor running a receiving web server on it's own thread that other clients can connect to to request blocks of data
 */
use std::net::{TcpListener, TcpStream};

use crate::config;

fn handle_client(mut stream: TcpStream) {
    todo!();
}

fn start_server() {
    //let mut mapping: HashMap<String, String> = HashMap::new();
    let listener = TcpListener::bind(config::TRANSFER_ADDRESS).unwrap();
    println!(
        "Distribution Server is listening on {}",
        config::TRANSFER_ADDRESS
    );

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream);
            }
            Err(e) => {
                eprintln!("Connection encountered an error: {}", e);
            }
        }
    }
}
