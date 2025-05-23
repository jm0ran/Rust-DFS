use std::collections::HashMap;
use std::collections::HashSet;
use std::io::Read;
use std::io::Write;
use std::net::{Shutdown, TcpStream};

#[derive(PartialEq)]
enum DiscoveryReadState {
    Initial,
    Targets,
    Complete,
}

/**
 * Constructs a request to send to the server
 * @param distributing: &Vec<String> - A vector of strings containing the files to distribute
 * @param requesting: &Vec<String> - A vector of strings containing the files to request
 * @param source: IP address of the source with the port, this isn't the IP connected to the listener server but the port used for file transfer between clients, broadcast address
 * @return Vec<String> - A vector of strings containing the request
 */
pub fn construct_request(
    distributing: &HashSet<String>,
    requesting: &HashSet<String>,
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
pub fn send_request(
    request: Vec<String>,
    linking_server: &str,
) -> Result<HashMap<String, HashSet<String>>, std::io::Error> {
    // Open the stream and send the request
    let mut stream: TcpStream = TcpStream::connect(linking_server)?;
    for line in request {
        stream.write_all(line.as_bytes())?;
    }

    // Our request is complete and we can shut down the write stream
    stream.shutdown(Shutdown::Write)?;

    // Read the response from the server
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    let result = process_response(response);

    return Ok(result);
}

/**
 * Process a response into a hashmap of hashes to a list of their providers
 * @param response: response as a string
 * @return results: HashMap of hashes to a list of their providers
 */
fn process_response(response: String) -> HashMap<String, HashSet<String>> {
    let mut results: HashMap<String, HashSet<String>> = HashMap::new();

    let mut read_state = DiscoveryReadState::Initial;
    for line in response.lines() {
        match read_state {
            DiscoveryReadState::Initial => {
                if line.starts_with("#S") {
                    read_state = DiscoveryReadState::Targets;
                }
            }
            DiscoveryReadState::Targets => {
                if line == "#T" {
                    continue; // Just want to pass over this for now
                } else if line == "#E" {
                    read_state = DiscoveryReadState::Complete;
                } else {
                    let mut line_parts = line.split(" ");
                    let file_hash: String = String::from(line_parts.next().unwrap());
                    let mut providers: HashSet<String> = HashSet::new();
                    for part in line_parts {
                        providers.insert(String::from(part));
                    }
                    results.insert(file_hash, providers);
                }
            }
            DiscoveryReadState::Complete => {
                continue; // Do noting for the moment
            }
        }
    }

    if read_state != DiscoveryReadState::Complete {
        panic!("Stream read state did not reach Complete State");
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
