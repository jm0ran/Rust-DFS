use std::{
    collections::{HashMap, HashSet},
    io::{BufRead, BufReader, Read, Write},
    net::{Shutdown, TcpStream},
    sync::{Arc, RwLock},
    thread::JoinHandle,
};

use crate::{config, file_ops};

/**
 * File builder class is in charge of building each individual file sending requests for blocks of data and then putting these blocks together
 */

enum FileStatus {
    Incomplete,
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
    output_file_path: String,
    block_states: Vec<BlockState>,
    file_hash: String,
    active_threads: HashMap<u64, JoinHandle<()>>,
    distributor_pool: HashSet<String>,
    distributors_in_use: HashSet<String>,
    distributors_available: HashSet<String>,
    blocks_remaining: u64,
    file_status: FileStatus,
    arc: Option<Arc<RwLock<FileBuilder>>>,
}

impl FileBuilder {
    /**
     * Construct a new file builder,
     */
    pub fn new(output_file_path: String, size: u64, file_hash: String) -> Arc<RwLock<FileBuilder>> {
        let total_blocks =
            (size / config::BLOCK_SIZE) + if size % config::BLOCK_SIZE != 0 { 1 } else { 0 };
        // Increment by 1 if non-standard size final block is necessary

        let _ = file_ops::reserve_file_space(&output_file_path, size).unwrap();

        let builder = FileBuilder {
            output_file_path,
            block_states: vec![BlockState::Waiting; total_blocks as usize],
            file_hash,
            active_threads: HashMap::new(),
            distributor_pool: HashSet::new(),
            distributors_in_use: HashSet::new(),
            distributors_available: HashSet::new(),
            blocks_remaining: total_blocks,
            file_status: FileStatus::Incomplete,
            arc: None,
        };

        // Store arc inside the object
        let arc = Arc::new(RwLock::new(builder));
        arc.write().unwrap().arc = Some(arc.clone());

        return arc.clone();
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
            println!("No Distributors Available");
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

    /**
     * The complete block function is called by the download thread upon completion of downloading and writing a block
     */
    pub fn complete_block(&mut self, block_num: u64, distributor: String) {
        // Mark the block as complete
        self.block_states[block_num as usize] = BlockState::Complete;
        self.blocks_remaining -= 1;

        // Remove the distributor from the in use pool, only add back into available if distributor is still in the larger pool
        self.distributors_in_use.remove(&distributor);
        if self.distributor_pool.contains(&distributor) {
            self.distributors_available.insert(distributor);
        }

        // Check if all blocks are complete
        if self.blocks_remaining == 0 {
            self.file_status = FileStatus::Complete;
            println!("File Download Complete");
            return;
        }
    }

    fn spawn_download_thread(&mut self, block_num: u64, distributor: String) {
        let request = self.create_block_request(block_num);
        let output_path = self.output_file_path.clone();
        let builder = self.arc.clone();
        self.active_threads.insert(
            block_num,
            std::thread::spawn(move || {
                // Connect to the target server and send our request
                let mut stream: TcpStream = TcpStream::connect(distributor.clone()).unwrap();
                for line in request {
                    stream.write_all(line.as_bytes()).unwrap();
                }
                stream.shutdown(Shutdown::Write).unwrap();

                let file_buffer = FileBuilder::process_response(&mut stream);

                println!("Requesting Block Num: {} From: {}", block_num, distributor);
                // Write buffer to file
                file_ops::write_block(&output_path, block_num, file_buffer).unwrap();

                // Call complete block function
                builder
                    .unwrap()
                    .write()
                    .unwrap()
                    .complete_block(block_num, distributor.clone());
            }),
        );
    }

    /**
     * Process the response string, this implementation is a little different from other reads across the program because we are not reading by line but instead by char
     * @todo Need to implement this
     */
    fn process_response(stream: &mut TcpStream) -> Vec<u8> {
        // Read the response from the server
        let mut buf_reader = BufReader::new(stream);

        // Read line 1
        let mut _line1 = String::new();
        buf_reader.read_line(&mut _line1).unwrap();
        _line1.trim().to_string();

        // Read line 2, extract data size
        let mut line2 = String::new();
        buf_reader.read_line(&mut line2).unwrap();
        line2 = line2.trim().to_string();
        let mut split = line2.split(' ');
        split.next(); // Skip the first part
        let block_size: u64 = split.next().unwrap().parse().unwrap();

        // Read the byte data
        let mut buffer = vec![0u8; block_size as usize];
        buf_reader.read_exact(&mut buffer).unwrap();

        // Read the final line
        let mut _line3 = String::new();
        buf_reader.read_line(&mut _line3).unwrap();
        _line3 = _line3.trim().to_string();

        // Return the buffer
        return buffer;
    }

    fn create_block_request(&self, block_num: u64) -> Vec<String> {
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
