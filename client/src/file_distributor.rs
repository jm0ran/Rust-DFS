/**
 * This will be our file distributor running a receiving web server on it's own thread that other clients can connect to to request blocks of data
 */
use std::{io::{BufRead, BufReader, Write}, net::{Shutdown, TcpListener, TcpStream}};

use crate::config;


#[derive(PartialEq)]
enum FileRequestReadState {
    Initial,
    Hash,
    BlockNum,
    Complete
}

fn handle_client(mut stream: TcpStream) {
    let reader = BufReader::new(&mut stream);
    
    // Collect our data from incoming stream
    let data: Vec<String> = reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    // Process data in request
    let mut read_state = FileRequestReadState::Initial;
    for line in data {
        match read_state {
            FileRequestReadState::Initial => {
                if line.starts_with("#S") {
                    read_state = FileRequestReadState::Hash;
                }
                continue;
            },
            FileRequestReadState::Hash => {
                if line.starts_with("#T"){
                    read_state = FileRequestReadState::BlockNum;
                }
                continue
            },
            FileRequestReadState::BlockNum => {
                if line.starts_with("#B"){
                    read_state = FileRequestReadState::Complete;
                }
            },
            FileRequestReadState::Complete => {
                continue;
            },
        }
    }

    if read_state != FileRequestReadState::Complete {
        panic!("File request read state did not reach COMPLETE state");
    }

    let response = String::from("Success\n");
    stream.write_all(response.as_bytes()).unwrap();

    stream.shutdown(Shutdown::Write).unwrap();
    println!("File Distribution Processed a successful connection");
}

pub fn start_server() {
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
