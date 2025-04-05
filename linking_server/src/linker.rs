/*
 * Primary responsibility of this file is to track who wants what at any given time
 * When web server is set up to be multithreaded it will be critical that this component is thread safe
 */

use std::{collections::HashMap, collections::HashSet, sync::OnceLock, sync::RwLock};

// OnceLock allows for lazy initialization and a singleton pattern, RW lock will ensure that multiple mutable references can be created while maintaining thread safety
static INSTANCE: OnceLock<RwLock<Linker>> = OnceLock::new();

pub struct Linker {
    /**
     * Distributing is a hashmap of the form <file_hash, distributing_addresses>
     */
    distributing: HashMap<String, HashSet<String>>,

    /**
     * Requesting is a hashmap of the form <file_hash, requesting_addresses>
     */
    requesting: HashMap<String, HashSet<String>>,
}

impl Linker {
    /**
     * Constructs a new Linker instance, private constructor to ensure that we only have one instance of the linker at a time
     * @return Linker - A new instance of the Linker struct
     */
    fn new() -> Self {
        Linker {
            distributing: HashMap::new(),
            requesting: HashMap::new(),
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
     * This function is used to register a clients distributing hashes with their corresponding addresses
     * @param source: String - The transfer server of the distributing address
     * @param distributing: Vec<String> - A vector of strings containing the hashes of the files being distributed
     * @return None
     */
    pub fn add_distributing(&mut self, source: String, distributing: Vec<String>) {
        for element in distributing {
            // Check if the element is already in the distributing hashmap
            if !self.distributing.contains_key(&element) {
                self.distributing.insert(element.clone(), HashSet::new());
            }

            // Add the address to this corresponding hashset
            self.distributing.get_mut(&element).unwrap().insert(source.clone());
        }

        // Print current state of distributing hashmap
        println!("\nDistributing Map:");
        for (key, value) in &self.distributing {
            println!("{}: {:?}", key, value);
        }
    }

    /**
     * This function is used to register a clients requesting hashes with their corresponding addresses
     * @param source: String - The transfer server of the requesting address
     * @param requesting: Vec<String> - A vector of strings containing the hashes of the files being requested
     * @return None
     */
    pub fn add_requesting(&mut self, source: String, requesting: Vec<String>) {
        for element in requesting {
            // Check if the element is already in the requesting hashmap
            if !self.requesting.contains_key(&element) {
                self.requesting.insert(element.clone(), HashSet::new());
            }

            // Add the address to this corresponding hashset
            self.requesting.get_mut(&element).unwrap().insert(source.clone());
        }
        // Print current state of requesting hashmap
        println!("\nRequesting Map:");
        for (key, value) in &self.requesting {
            println!("{}: {:?}", key, value);
        }
    }
}
