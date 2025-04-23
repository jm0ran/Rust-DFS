use crate::{config, file_manager, linker_comm};
/**
 * Maintains communication with linking server
 */
use std::{
    collections::{HashMap, HashSet},
    sync::{OnceLock, RwLock},
};

static INSTANCE: OnceLock<RwLock<Linker>> = OnceLock::new();

pub struct Linker {
    target: String,
    distributors: HashMap<String, HashSet<String>>,
}

impl Linker {
    /**
     * Private constructor for singleton pattern
     */
    fn new() -> Self {
        Linker {
            distributors: HashMap::new(),
            target: String::from(config::DEFAULT_LINKER),
        }
    }

    /**
     * This is for the singleton pattern and ensures that we only have one instance of the linker at a time and that this instance can be requested globally
     * @return &'static RwLock<Linker> - A reference to the linker instance
     */
    pub fn instance() -> &'static RwLock<Linker> {
        INSTANCE.get_or_init(|| RwLock::new(Linker::new()))
    }

    /**
     * Return target of linker as a string representing linker server address
     */
    pub fn get_target(&self) -> String {
        return self.target.clone();
    }

    /**
     * Set the target of linker with updated string
     */
    pub fn set_target(&mut self, new_target: String) {
        self.target = new_target;
    }

    /**
     * Update the linker by making a request to the linking server
     * @todo update arguments, right now just linking up some existing code
     * @todo will need some level of error handling here for if server is unable to be reached
     */
    pub fn update(&mut self) -> Option<std::io::Error> {
        // Get requesting and distributing from file manager
        let distributing: HashSet<String>;
        let requesting: HashSet<String>;
        {
            let file_manager = file_manager::FileManager::instance();
            let lock = file_manager.read().unwrap();
            distributing = lock.get_distributing_hashes();
            requesting = lock.get_requesting_hashes();
        }

        let request = linker_comm::construct_request(
            &distributing,
            &requesting,
            String::from(config::TRANSFER_ADDRESS),
        );
        match linker_comm::send_request(request, &self.target) {
            Ok(result) => {
                self.distributors = result;
            }
            Err(error) => {
                return Some(error);
            }
        }

        // Send distribution information to file manager
        self.send_distributors();

        return None;
    }

    /**
     * Send distributors to file_manager to be distributed across the builders, we don't want to request the distributors from the file manager as this could result in deadlock
     */
    fn send_distributors(&self) {
        // Get the file manager
        let file_manager = file_manager::FileManager::instance();
        // Get a write lock on the file manager
        let mut lock = file_manager.write().unwrap();
        // Send the distributors to the file manager
        lock.set_distributors(self.distributors.clone());
    }

    /**
     * Debug function to see values in the linker
     */
    pub fn debug(&self) {
        println!("Distributors:\n{:?}", self.distributors);
        // Get requesting and distributing from file manager
        let distributing: HashSet<String>;
        let requesting: HashSet<String>;
        {
            let file_manager = file_manager::FileManager::instance();
            let lock = file_manager.read().unwrap();
            distributing = lock.get_distributing_hashes();
            requesting = lock.get_requesting_hashes();
        }
        println!(
            "Distributing: {:?}\nRequesting: {:?}",
            distributing, requesting
        );
    }
}
