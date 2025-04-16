/**
 * This will be our file distributor running a receiving web server on it's own thread that other clients can connect to to request blocks of data
 * @todo: Lot of unwraps here that I should look into
 */
use std::{
    io::{BufRead, BufReader, Write},
    net::{Shutdown, TcpListener, TcpStream},
};

use crate::{config, file_manager, file_ops};

#[derive(PartialEq)]
enum FileRequestReadState {
    Initial,
    Hash,
    BlockNum,
    Complete,
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
    let mut block_num: u64 = 0;
    let mut file_hash: String = String::from("");
    for line in data {
        match read_state {
            FileRequestReadState::Initial => {
                if line.starts_with("#S") {
                    read_state = FileRequestReadState::Hash;
                }
                continue;
            }
            FileRequestReadState::Hash => {
                if line.starts_with("#T") {
                    read_state = FileRequestReadState::BlockNum;
                    let mut line_parts = line.split(' ');
                    let _ = line_parts.next(); // Throw out first part
                    file_hash = String::from(line_parts.next().unwrap());
                }
                continue;
            }
            FileRequestReadState::BlockNum => {
                if line.starts_with("#B") {
                    read_state = FileRequestReadState::Complete;
                    let mut line_parts = line.split(' ');
                    let _ = line_parts.next(); // Throw out first part
                    block_num = line_parts.next().unwrap().parse::<u64>().unwrap();
                }
            }
            FileRequestReadState::Complete => {
                continue;
            }
        }
    }

    if read_state != FileRequestReadState::Complete {
        panic!("File request read state did not reach COMPLETE state");
    }

    // Get File Path
    let file_path = file_manager::FileManager::instance()
        .read()
        .unwrap()
        .get_path_from_hash(&file_hash);
    if file_path == "" {
        panic!("A file path was not found to gather block from");
    }
    let block = file_ops::get_block(file_path, block_num).unwrap();

    // Construct Response
    let mut response: Vec<Vec<u8>> = Vec::new();
    response.push(format!("#S RDFS 0.1 BLOCK_REQUEST {}\n", config::BLOCK_SIZE).into_bytes());
    response.push(format!("#D {}\n", block.len()).into_bytes());
    response.push(block);
    response.push(format!("#E\n").into_bytes());

    // Write the response
    for part in response {
        stream.write_all(&part).unwrap();
    }

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
