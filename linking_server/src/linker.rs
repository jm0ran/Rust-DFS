/*
 * Primary responisibility of this file is to track who wants what at any given time
 * When web server is set up to be multithreaded it will be critical that this component is thread safe
 */

use std::{collections::HashMap, sync::OnceLock, sync::RwLock};

// OnceLock allows for lazy initialization and a singleton pattern, RW lock will ensure that mutliple mutable references can be created while maintaining thread safety
static INSTANCE: OnceLock<RwLock<Linker>> = OnceLock::new();

pub struct Linker {
    distributing: HashMap<String, String>,
    requesting: HashMap<String, String>,
}

impl Linker {
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

    pub fn add_distributing(&mut self, source: &str, hash: &str){
        println!("Added Distributing: source: {source} with hash: {hash}");
    }

    pub fn add_requesting(&mut self, source: &str, hash: &str){
        println!("Added Requesting: source: {source} with hash: {hash}");
    }
}
