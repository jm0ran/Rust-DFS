mod linker;

use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

#[derive(Debug, PartialEq)]
enum StreamReadState {
    INITIAL,
    ADDRESS,
    DISTRIBUTING,
    REQUESTING,
    COMPLETE,
}

fn main() {
    run_server();
}

fn run_server() {
    //let mut mapping: HashMap<String, String> = HashMap::new();
    let listener = TcpListener::bind("127.0.0.1:7777").unwrap();
    println!("Linking Server is listening on localhost port 7777");

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

fn get_response(distributing: Vec<String>, requesting: Vec<String>, source: String) -> String {
    let linker = linker::Linker::instance();

    // First we want to register this client as a supplier for the files it is distributing
    {
        // Get our write lock on the linker
        let mut linker_lock = linker.write().unwrap();
        linker_lock.add_distributing(source.clone(), distributing);
        linker_lock.add_requesting(source.clone(), requesting);
    }

    return format!("HTTP/1.1 200 OK\r\n\r\nReceived Data\r\n");
}

fn handle_client(mut stream: TcpStream) {
    // Create vectors for our distributing and requesting data
    let mut distributing: Vec<String> = Vec::new();
    let mut requesting: Vec<String> = Vec::new();
    let mut address: String = String::new();

    //Collect our data from incoming stream
    let reader = BufReader::new(&mut stream);
    let data: Vec<String> = reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    // Process data into our Vectors
    let mut read_state = StreamReadState::INITIAL;
    for line in data {
        match read_state {
            StreamReadState::INITIAL => {
                // Todo: Check state of request header to ensure protocol is correct, for now assume it is and more on
                if line.starts_with("#S") {
                    read_state = StreamReadState::ADDRESS;
                }
                continue;
            }
            StreamReadState::ADDRESS => {
                if line.starts_with("#A") {
                    // Parse out the address
                    address.push_str(line.split_at(3).1);
                    read_state = StreamReadState::DISTRIBUTING;
                }
                continue;
            }
            StreamReadState::DISTRIBUTING => {
                if line == "#D" {
                    // We don't need to capture this line
                    continue;
                }
                if line == "#R" {
                    read_state = StreamReadState::REQUESTING;
                } else {
                    distributing.push(line.clone());
                }
                continue;
            }
            StreamReadState::REQUESTING => {
                if line == "#E" {
                    read_state = StreamReadState::COMPLETE;
                } else {
                    requesting.push(line.clone());
                }
                continue;
            }
            StreamReadState::COMPLETE => {
                continue;
            }
        }
    }

    if read_state != StreamReadState::COMPLETE {
        panic!("Stream read state did not reach COMPLETE state");
    }

    // Send our response indicating completion
    let response = get_response(distributing, requesting, address.clone());
    stream.write_all(response.as_bytes()).unwrap();

    println!("Processed Request From: {}", &address);
}
