mod linker;

use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

#[derive(Debug, PartialEq)]
enum DiscoveryReadState {
    INITIAL,
    ADDRESS,
    DISTRIBUTING,
    REQUESTING,
    COMPLETE,
}

// This is the port that the server will listen on for incoming requests
static SERVER_ADDRESS: &str = "127.0.0.1:7771";

fn main() {
    run_server();
}

fn run_server() {
    //let mut mapping: HashMap<String, String> = HashMap::new();
    let listener = TcpListener::bind(SERVER_ADDRESS).unwrap();
    println!("Linking Server is listening on {SERVER_ADDRESS}");

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

fn get_response(distributing: Vec<String>, requesting: Vec<String>, source: String) -> Vec<String> {
    let linker = linker::Linker::instance();

    // First we want to register this client as a supplier for the files it is distributing
    {
        // Get our write lock on the linker
        let mut linker_lock = linker.write().unwrap();
        linker_lock.add_distributing(source.clone(), &distributing);
        linker_lock.add_requesting(source.clone(), &requesting);
    }

    let distributors: HashMap<String, Vec<String>>; // This will be the response we send back to the client
                                                    // Next we wan to open a read lock on the linker and get the distributors for the files we are requesting
    {
        // Get our read lock on the linker
        let linker_lock = linker.read().unwrap();
        distributors = linker_lock.get_distributors(&requesting);
    }

    // Construct the response
    let mut response: Vec<String> = Vec::new();
    response.push(String::from("#S RDFS 0.1 DISCOVERY_POST\n"));
    response.push(String::from("#T\n"));
    for entry in distributors.iter() {
        let mut line: String = format!("{}", entry.0);
        for provider in entry.1 {
            line.push_str(format!(" {}", provider).as_str());
        }
        line.push('\n');
        response.push(line);
    }
    response.push(String::from("#E\n"));

    return response;
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
    let mut read_state = DiscoveryReadState::INITIAL;
    for line in data {
        match read_state {
            DiscoveryReadState::INITIAL => {
                // Todo: Check state of request header to ensure protocol is correct, for now assume it is and more on
                if line.starts_with("#S") {
                    read_state = DiscoveryReadState::ADDRESS;
                }
                continue;
            }
            DiscoveryReadState::ADDRESS => {
                if line.starts_with("#A") {
                    // Parse out the address
                    address.push_str(line.split_at(3).1);
                    read_state = DiscoveryReadState::DISTRIBUTING;
                }
                continue;
            }
            DiscoveryReadState::DISTRIBUTING => {
                if line == "#D" {
                    // We don't need to capture this line
                    continue;
                }
                if line == "#R" {
                    read_state = DiscoveryReadState::REQUESTING;
                } else {
                    distributing.push(line.clone());
                }
                continue;
            }
            DiscoveryReadState::REQUESTING => {
                if line == "#E" {
                    read_state = DiscoveryReadState::COMPLETE;
                } else {
                    requesting.push(line.clone());
                }
                continue;
            }
            DiscoveryReadState::COMPLETE => {
                continue;
            }
        }
    }

    if read_state != DiscoveryReadState::COMPLETE {
        panic!("Stream read state did not reach COMPLETE state");
    }

    // Send our response indicating completion
    let response = get_response(distributing, requesting, address.clone());

    // Write response into the stream
    for line in response {
        stream.write_all(line.as_bytes()).unwrap();
    }

    println!("Processed Request From: {}", &address);
}
