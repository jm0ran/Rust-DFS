use std::io::Write;
use std::net::TcpStream;

mod file_ops;

const TARGET: &str = "127.0.0.1:7777";

/**
 * Constructs a request to send to the server
 * @param distributing: &Vec<String> - A vector of strings containing the files to distribute
 * @param requesting: &Vec<String> - A vector of strings containing the files to request
 * @return Vec<String> - A vector of strings containing the request
 */
fn construct_request(distributing: &Vec<String>, requesting: &Vec<String>) -> Vec<String> {
    let mut response: Vec<String> = Vec::new();
    response.push(String::from("#S RDFS 0.1 DISCOVERY_POST\n"));

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
    let mut stream: TcpStream = TcpStream::connect(TARGET).unwrap();
    for line in request {
        stream.write_all(line.as_bytes()).unwrap();
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

    // Construct the request
    let request = construct_request(&distributing, &requesting);

    // Send the request
    send_request(request);
}
