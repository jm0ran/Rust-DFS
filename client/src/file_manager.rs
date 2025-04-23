use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::{Arc, OnceLock, RwLock},
};

use crate::{
    config,
    file_builder::{self, FileBuilder},
    file_ops,
};

static INSTANCE: OnceLock<RwLock<FileManager>> = OnceLock::new();

pub struct FileManager {
    distributing: HashMap<String, String>, //Hash map of file path to file hash
    requesting: HashMap<String, String>,   // Hash map of file path to file hash
    builders: HashMap<String, Arc<RwLock<file_builder::FileBuilder>>>, // Map of hash to builder
}

impl FileManager {
    /**
     * Private constructor for singleton pattern
     */
    fn new() -> Self {
        FileManager {
            distributing: HashMap::new(),
            requesting: HashMap::new(),
            builders: HashMap::new(),
        }
    }

    pub fn instance() -> &'static RwLock<FileManager> {
        INSTANCE.get_or_init(|| RwLock::new(FileManager::new()))
    }

    /**
     * Scan the distributing files and update their values in the file manager
     */
    pub fn scan_distributing(&mut self) -> Option<std::io::Error> {
        match file_ops::hash_files_shallow(config::DISTRIBUTING_PATH) {
            Ok(result) => {
                self.distributing = result;
            }
            Err(err) => {
                return Some(err);
            }
        }
        return None;
    }

    /**
     * Get distributing files
     * @return a hash map of file_path: file_hash
     */
    pub fn get_distributing(&self) -> HashMap<String, String> {
        return self.distributing.clone();
    }

    /**
     * Return distributing hashes, used by linker
     */
    pub fn get_distributing_hashes(&self) -> HashSet<String> {
        // This is a very inefficient way to get hash set of hashes, will improve later
        let mut distributing_hashes = HashSet::new();
        for entry in self.distributing.iter() {
            distributing_hashes.insert(String::from(entry.1));
        }
        return distributing_hashes;
    }

    /**
     * Return requesting hashes, used by linker
     */
    pub fn get_requesting_hashes(&self) -> HashSet<String> {
        // This is a very inefficient way to get hash set of hashes, will improve later
        let mut requesting_hashes = HashSet::new();
        for entry in self.requesting.iter() {
            requesting_hashes.insert(String::from(entry.1));
        }
        return requesting_hashes;
    }

    /**
     * Given a hash return the associated file path to read from
     */
    pub fn get_path_from_hash(&self, target_hash: &str) -> String {
        // @todo: Should probably improve this from linear complexity
        for entry in &self.distributing {
            let (path, hash) = entry;
            if hash == target_hash {
                return path.clone();
            }
        }
        return String::from("");
    }

    /**
     * Register files to request, will only register if this file is not already in the requesting list
     * @param Request path
     */
    pub fn register_requesting(&mut self, request_path: String) -> Result<(), std::io::Error> {
        // Check if the file is already in the requesting list
        if self.requesting.contains_key(&request_path) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "File is already in the requesting list",
            ));
        }

        // Get lines from request file and process
        let mut lines = file_ops::get_raw_lines(&request_path)?;

        // Process First Line
        let mut first_line = lines.pop().ok_or(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to read first line from request file",
        ))?;
        first_line = first_line.trim().to_string(); // Remove any whitespace
        let mut line_parts = first_line.split(" ");
        line_parts.next(); // Skip the Line identifier
        line_parts.next(); // Skip the block size, for now...
        let file_size: u64 = line_parts
            .next()
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to read file size from request file",
            ))?
            .parse()
            .map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to parse file size from request file",
                )
            })?;
        let file_hash = line_parts.next().ok_or(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to read file hash from request file",
        ))?;

        // Read files blocks
        let mut hashes: Vec<String> = Vec::new();
        let mut next_line;
        let mut remaining = true;
        while remaining {
            next_line = lines.pop().ok_or(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Request file did not terminate with #E",
            ))?;
            next_line = next_line.trim().to_string();
            remaining = !next_line.starts_with("#E");
            match next_line.starts_with("#E") {
                true => {
                    remaining = false;
                    let mut parts = next_line.split(" ");
                    parts.next(); // We don't care about first piece
                    let hash = String::from(parts.next().ok_or(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Failed to extract hash from the final line",
                    ))?);
                    hashes.push(hash);
                }
                false => {
                    hashes.push(next_line);
                }
            }
        }

        // Prep output file path name
        let mut output_path = Path::new(&request_path)
            .file_name()
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Could not convert request path string to path",
            ))?
            .to_str()
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Could not get filename from request file path",
            ))?;
        output_path = &output_path[..output_path.len() - 5];
        let formatted_output_path = format!("./receiving/{}", output_path);
        output_path = &formatted_output_path;

        let file_builder = FileBuilder::new(
            String::from(output_path),
            file_size,
            String::from(file_hash),
        );
        file_builder.write().unwrap().start_next_block();
        self.builders.insert(file_hash.to_string(), file_builder);
        self.requesting
            .insert(request_path.clone(), file_hash.to_string());
        return Ok(());
    }

    /**
     * Called by linker to pass distributors, file manager is responsible for sending them all to the correct file_builder
     */
    pub fn set_distributors(&mut self, distributors: HashMap<String, HashSet<String>>) {
        println!("Set Distributors: {:?}", distributors);
        for entry in &distributors {
            let hash = entry.0;
            let i_distributors = entry.1;
            if self.builders.contains_key(hash) {
                let builder = self.builders.get_mut(hash).unwrap();
                builder
                    .write()
                    .unwrap()
                    .add_distributors(i_distributors.clone());
                // Start a block for every distributor
                for _i in 0..i_distributors.len() {
                    println!("Started a block");
                    builder.write().unwrap().start_next_block();
                }
            }
        }
    }
}
