mod linker;

use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

#[derive(Debug, PartialEq)]
enum StreamReadState {
    INITIAL,
    DISTRIBUTING,
    REQUESTING,
    COMPLETE,
}

fn main() {
    run_server();
}

fn run_server() {
    // Get our linker
    let linker = linker::Linker::instance();

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
        let mut linkerLock = linker.write().unwrap();
        
        // Add our distributing data to the linker
        for entry in distributing.iter() {
            linkerLock.add_distributing(&source, entry);
        }

        // Add our requesting data to the linker
        for entry in requesting.iter() {
            linkerLock.add_requesting(&source, entry);
        }
    }

    return format!("HTTP/1.1 200 OK\r\n\r\nReceived Data\r\n");
}

fn handle_client(mut stream: TcpStream) {
    // Create vectors for our distributing and requesting data
    let mut distributing: Vec<String> = Vec::new();
    let mut requesting: Vec<String> = Vec::new();

    // Get our source address
    let source = stream.peer_addr().unwrap().to_string();
    
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
                if line == "#D" {
                    read_state = StreamReadState::DISTRIBUTING;
                }
                continue;
            }
            StreamReadState::DISTRIBUTING => {
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
    let response = get_response(distributing, requesting, source);
    stream.write_all(response.as_bytes()).unwrap();
    println!("Processed Request");
}
