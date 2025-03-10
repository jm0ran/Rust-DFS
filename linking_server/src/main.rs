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

fn handle_client(mut stream: TcpStream) -> (Vec<String>, Vec<String>) {
    // Create vectors for our distributing and requesting data
    let mut distributing: Vec<String> = Vec::new();
    let mut receiving: Vec<String> = Vec::new();

    //Collect our data from incoming stream
    let reader = BufReader::new(&mut stream);
    let data: Vec<String> = reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    // Process data into our Vectors
    println!("-----------------------------");
    let mut read_state = StreamReadState::INITIAL;
    for line in data {
        println!("State: {:#?} Line {line}", read_state);
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
                    receiving.push(line.clone());
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
    let response = format!("HTTP/1.1 200 OK\r\n\r\nReceived Data\r\n");
    stream.write_all(response.as_bytes()).unwrap();
    return (distributing, receiving);
}
