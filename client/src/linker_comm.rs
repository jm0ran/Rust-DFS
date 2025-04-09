use crate::file_ops;
use std::collections::HashMap;
use std::io::Read;
use std::io::Write;
use std::net::{Shutdown, TcpStream};

const LINKING_SERVER: &str = "127.0.0.1:7771";
const TRANSFER_SERVER: &str = "127.0.0.1:7779";

enum DiscoveryReadState {
    INITIAL,
    TARGETS,
    COMPLETE,
}

/**
 * Constructs a request to send to the server
 * @param distributing: &Vec<String> - A vector of strings containing the files to distribute
 * @param requesting: &Vec<String> - A vector of strings containing the files to request
 * @param source: IP address of the source with the port, this isn't the IP connected to the listener server but the port used for file transfer between clients, broadcast address
 * @return Vec<String> - A vector of strings containing the request
 */
fn construct_request(
    distributing: &Vec<String>,
    requesting: &Vec<String>,
    source: String,
) -> Vec<String> {
    let mut response: Vec<String> = Vec::new();
    response.push(String::from("#S RDFS 0.1 DISCOVERY_POST\n"));

    // Add the source IP address to the request
    response.push(format!("#A {}\n", source));

    // Add all distributing files to the request
    response.push(String::from("#D\n"));
    for s in distributing {
        response.push(format!("{}\n", s.clone()));
    }

    // Add all requesting files to the request
    response.push(String::from("#R\n"));
    for s in requesting {
        response.push(format!("{}\n", s.clone()));
    }

    // Indicate end of transmission
    response.push(String::from("#E\n"));

    return response;
}

/**
 * Sends a request to the server
 * @param request: Vec<String> - A vector of strings containing the request
 * @return None
 */
fn send_request(request: Vec<String>) {
    // Open the stream and send the request
    let mut stream: TcpStream = TcpStream::connect(LINKING_SERVER).unwrap();
    for line in request {
        stream.write_all(line.as_bytes()).unwrap();
    }

    // Our request is complete and we can shut down the write stream
    stream.shutdown(Shutdown::Write).unwrap();

    // Read the response from the server
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    let result = process_response(response);

    // Print the results for debugging temporarily
    println!("Results:");
    for (file_hash, providers) in result {
        println!("File Hash: {}", file_hash);
        for provider in providers {
            println!("\tProvider: {}", provider);
        }
    }
}

/**
 * Process a response into a hashmap of hashes to a list of their providers
 * @param response: response as a string
 * @return results: HashMap of hashes to a list of their providers
 */
fn process_response(response: String) -> HashMap<String, Vec<String>> {
    let mut results: HashMap<String, Vec<String>> = HashMap::new();

    let mut read_state = DiscoveryReadState::INITIAL;
    for line in response.lines() {
        match read_state {
            DiscoveryReadState::INITIAL => {
                if line.starts_with("#S") {
                    read_state = DiscoveryReadState::TARGETS;
                }
            }
            DiscoveryReadState::TARGETS => {
                if line == "#T" {
                    continue; // Just want to pass over this for now
                } else if line == "#E" {
                    read_state = DiscoveryReadState::COMPLETE;
                } else {
                    let mut line_parts = line.split(" ");
                    let file_hash: String = String::from(line_parts.next().unwrap());
                    let mut providers: Vec<String> = Vec::new();
                    for part in line_parts {
                        providers.push(String::from(part));
                    }
                    results.insert(file_hash, providers);
                }
            }
            DiscoveryReadState::COMPLETE => {
                continue; // Do noting for the moment
            }
        }
    }

    return results;
}

/** For debugging */
#[allow(dead_code)]
fn print_request(request: &Vec<String>) {
    println!("Sent Request:");
    for line in request {
        print!("{}", line);
    }
}

pub fn run() {
    // Get files to distribute
    let distributing: Vec<String> = file_ops::hash_files_shallow("temp")
        .values()
        .cloned()
        .collect();

    // Create a temporary requesting array
    let mut requesting: Vec<String> = Vec::new();
    requesting.push(String::from("2dd184b8c84b999a6ccc7ae4da2efc3b3cd455d50a04686caaf90f8f5cd60194c8e0e758947738f1001e01010ddb28e782ed274c966561ba798fe0123f495b5d"));
    requesting.push(String::from("FAKE_HASH"));

    // Construct the request
    let request = construct_request(&distributing, &requesting, TRANSFER_SERVER.to_string());

    // Send the request
    send_request(request);
}
