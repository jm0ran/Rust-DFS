use crate::linker_comm;
/**
 * Maintains communication with linking server
 */
use std::sync::{OnceLock, RwLock};

static INSTANCE: OnceLock<RwLock<Linker>> = OnceLock::new();

pub struct Linker {
    target: String,
}

impl Linker {
    /**
     * Private constructor for singleton pattern
     */
    fn new() -> Self {
        Linker {
            target: String::from(""),
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
    pub fn update(&mut self) {
        linker_comm::run();
    }
}
