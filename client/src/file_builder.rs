use std::{
    collections::{HashMap, HashSet},
    io::{Read, Write},
    net::{Shutdown, TcpStream},
    thread::JoinHandle,
};

use crate::config;

/**
 * File builder class is in charge of building each individual file sending requests for blocks of data and then putting these blocks together
 */

enum FileStatus {
    AwaitingDistributor,
    AwaitingSize,
    InProgress,
    Complete,
}

#[derive(Clone, PartialEq)]
enum BlockState {
    Waiting,
    InProgress,
    Complete,
}

/**
 * @todo Going to need some big thought behind how I want to synchronize this, not going to stress too much right at this moment
 */
pub struct FileBuilder {
    request_file_path: String,
    output_file_path: String,
    block_states: Vec<BlockState>,
    file_hash: String,
    active_threads: HashMap<u64, JoinHandle<()>>,
    distributor_pool: HashSet<String>,
    distributors_in_use: HashSet<String>,
    distributors_available: HashSet<String>,
    blocks_remaining: u64,
}

impl FileBuilder {
    /**
     * Construct a new file builder,
     */
    pub fn new(
        request_file_path: String,
        output_file_path: String,
        size: u64,
        file_hash: String,
    ) -> FileBuilder {
        let total_blocks =
            (size / config::BLOCK_SIZE) + if size % config::BLOCK_SIZE != 0 { 1 } else { 0 };
        // Increment by 1 if non-standard size final block is necessary

        FileBuilder {
            request_file_path,
            output_file_path,
            block_states: vec![BlockState::Waiting; total_blocks as usize],
            file_hash,
            active_threads: HashMap::new(),
            distributor_pool: HashSet::new(),
            distributors_in_use: HashSet::new(),
            distributors_available: HashSet::new(),
            blocks_remaining: total_blocks,
        }
    }

    /**
     * Register new distributors for this file
     * 3 collections used as if a distributor is no longer recognized by the linker it should still be in use but should not return to the available pool
     */
    pub fn add_distributors(&mut self, distributors: HashSet<String>) {
        for distributor in &distributors {
            if self.distributors_in_use.contains(distributor)
                || self.distributors_available.contains(distributor)
            {
                continue;
            } else {
                self.distributors_available.insert(distributor.clone());
            }
        }
        self.distributor_pool = distributors;
    }

    /**
     * Look for the next block to start
     */
    pub fn start_next_block(&mut self) {
        if !(self.distributors_available.len() > 0) {
            // If there are no distributors available do not start the next block
            return;
        }
        for i in 0..self.block_states.len() {
            if self.block_states[i] == BlockState::Waiting {
                self.block_states[i] = BlockState::InProgress;
                // Get a distributor
                let distributor = self.distributors_available.iter().next().cloned().unwrap(); //Already verified there will be a distributor, if it crashes then theres an issue somewhere else
                self.distributors_in_use.insert(distributor.clone());
                self.spawn_download_thread(i as u64, distributor);
            }
        }
    }

    pub fn spawn_download_thread(&mut self, block_num: u64, distributor: String) {
        let request = self.create_block_request(block_num);
        self.active_threads.insert(
            block_num,
            std::thread::spawn(move || {
                // Connect to the target server and send our request
                let mut stream: TcpStream = TcpStream::connect(distributor).unwrap();
                for line in request {
                    stream.write_all(line.as_bytes()).unwrap();
                }
                stream.shutdown(Shutdown::Write).unwrap();

                // Read the response from the server
                let mut response = String::new();
                stream.read_to_string(&mut response).unwrap();
                println!("Response: {response}");
            }),
        );
    }

    pub fn create_block_request(&self, block_num: u64) -> Vec<String> {
        let mut request: Vec<String> = Vec::new();
        request.push(format!(
            "#S RDFS 0.1 BLOCK_REQUEST {}\n",
            config::BLOCK_SIZE
        ));
        request.push(format!("#T {}\n", self.file_hash));
        request.push(format!("#B {}\n", block_num));
        request.push(format!("#E"));
        return request;
    }

    pub fn read_file() {
        todo!();
    }
}
