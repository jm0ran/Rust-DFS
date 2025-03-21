/*
 * Primary responisibility of this file is to track who wants what at any given time
 * When web server is set up to be multithreaded it will be critical that this component is thread safe
 */

use std::{collections::HashMap, collections::HashSet, sync::OnceLock, sync::RwLock};

// OnceLock allows for lazy initialization and a singleton pattern, RW lock will ensure that mutliple mutable references can be created while maintaining thread safety
static INSTANCE: OnceLock<RwLock<Linker>> = OnceLock::new();

pub struct Linker {
    distributing: HashMap<String, HashSet<String>>,
    requesting: HashMap<String, HashSet<String>>,
}

impl Linker {
    fn new() -> Self {
        Linker {
            distributing: HashMap::new(),
            requesting: HashMap::new(),
        }
    }

    pub fn instance() -> &'static RwLock<Linker> {
        INSTANCE.get_or_init(|| RwLock::new(Linker::new()))
    }

    pub fn add_distributing(&mut self, source: String, elements: Vec<String>) {
        println!("\nReceived from {}", source);
        if !self.distributing.contains_key(&source){
            self.distributing.insert(source.clone(), HashSet::new());
        }
        for element in elements {
            self.distributing.get_mut(&source).unwrap().insert(element);
        }

        // Print current state of distributing hashmap
        for (key, value) in &self.distributing {
            println!("{}: {:?}", key, value);
        }
    }

    pub fn add_requesting(&mut self, source: String, elements: Vec<String>) {
        println!("\nReceived from {}", source);
        if !self.requesting.contains_key(&source){
            self.requesting.insert(source.clone(), HashSet::new());
        }
        for element in elements {
            self.requesting.get_mut(&source).unwrap().insert(element);
        }

        // Print current state of requesting hashmap
        for (key, value) in &self.requesting {
            println!("{}: {:?}", key, value);
        }
    }
}
