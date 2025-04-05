use std::io::Read;
use std::io::Write;
use std::net::{Shutdown, TcpStream};

mod file_ops;

const LINKING_SERVER: &str = "127.0.0.1:7771";
const TRANSFER_SERVER: &str = "127.0.0.1:7779";

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
    println!("Server Response: {}", response);
}

/** For debugging */
#[allow(dead_code)]
fn print_request(request: &Vec<String>) {
    println!("Sent Request:");
    for line in request {
        print!("{}", line);
    }
}

fn main() {
    // Get files to distribute
    let distributing: Vec<String> = file_ops::hash_files_shallow("temp")
        .values()
        .cloned()
        .collect();

    // Create a temporary requesting array
    let mut requesting: Vec<String> = Vec::new();
    requesting.push(String::from("TEMPORARY REQUEST VALUE"));
    requesting.push(String::from("TEMPORARY REQUEST VALUE 2"));

    // Construct the request
    let request = construct_request(&distributing, &requesting, TRANSFER_SERVER.to_string());

    // Send the request
    send_request(request);
}
