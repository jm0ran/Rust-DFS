use std::{
    collections::HashMap,
    sync::{OnceLock, RwLock},
};

use crate::{config, file_ops};

static INSTANCE: OnceLock<RwLock<FileManager>> = OnceLock::new();

pub struct FileManager {
    distributing: HashMap<String, String>, //Hash map of file path to file hash
}

impl FileManager {
    /**
     * Private constructor for singleton pattern
     */
    fn new() -> Self {
        FileManager {
            distributing: HashMap::new(),
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
}
