use std::{
    collections::{HashMap, HashSet},
    sync::{OnceLock, RwLock},
};

use crate::{config, file_builder, file_ops};

static INSTANCE: OnceLock<RwLock<FileManager>> = OnceLock::new();

pub struct FileManager {
    distributing: HashMap<String, String>, //Hash map of file path to file hash
    requesting: HashMap<String, String>,   // Hash map of file path to file hash
    receiving_builders: HashMap<String, file_builder::FileBuilder>,
}

impl FileManager {
    /**
     * Private constructor for singleton pattern
     */
    fn new() -> Self {
        FileManager {
            distributing: HashMap::new(),
            requesting: HashMap::new(),
            receiving_builders: HashMap::new(),
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
        //@todo: update the linker
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
     * Register files to request
     * @param requesting: Tuple of (file_path, file_hash)
     */
    pub fn register_requesting(&mut self, requesting: (String, String)) {
        // Want to rework this to process request files
        return;
        todo!("Not currently implemented");

        // let path = requesting.0;
        // let hash = requesting.1;

        // match self.requesting.get(&path) {
        //     Some(found_hash) => {
        //         if &hash != found_hash {
        //             // Needs to be overwritten as hash to search for has change
        //             println!("Warning: New request does not match previously stored hash");
        //             // @todo: Figure out how I want to implement this case, isn't super pressing at the moment
        //         }
        //     }
        //     None => {
        //         // This is not yet registered
        //         self.requesting.insert(path.clone(), hash.clone());
        //         self.receiving_builders
        //             .insert(path.clone(), file_builder::FileBuilder::new(hash));
        //     }
        // }
    }
}
