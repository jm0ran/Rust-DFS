/**
 * Plain Text User Interface For The Client
 */
use std::io::{self, Write};
use std::str::Split;
use crate::linker;

fn linker_cmd(mut line_parts: Split<'_, &str>) {
    // Get reference to the linker singleton
    let linker = linker::Linker::instance();
    
    // Ensure line has a next argument
    let arg_1;
    match line_parts.next() {
        Some(arg) => {
            arg_1 = arg;
        },
        None => {
            println!("Did not provide expect arguments for linker command");
            return;
        },
    }
    
    match arg_1 {
        "set" => {
            let new_target: String;
            match line_parts.next() {
                Some(target) => {
                    new_target = String::from(target); // Will want some input validation here eventually
                },
                None => {
                    println!("Did not provide a new linking server address to set");
                    return;
                }
            }

            {
                let mut lock = linker.write().unwrap();
                lock.set_target(new_target);
            }
        },
        "get" => {
            let target:String;
            {
                let lock = linker.read().unwrap();
                target = lock.get_target();
            }
            println!("Target: {target}")
        },
        _ => {
            println!("Unrecognized Linker Argument: {}", arg_1);
        },
    }
    
}

/**
 * Process input and pass to the appropriate command
 */
pub fn process_input(line: &str) {
    let mut line_parts = line.split(" ");
    if line_parts.clone().count() < 1 {
        // If we do not have an initial command
        println!("Unrecognized command: {}", line);
        return;
    }

    let arg_1 = line_parts.next().unwrap();

    match arg_1 {
        "exit" => {
            println!("Exiting...");
            std::process::exit(0);
        },
        "linker" => {
            println!("Linker");
            linker_cmd(line_parts);
        },
        _ => {
            println!("Unrecognized command: {}", line);
        }
    }
}

#[allow(while_true)] // Allowing while true for constant input loop
pub fn run() {
    let mut input = String::new();
    while true {
        print!("~ ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();
        input = input.to_ascii_lowercase(); // convert to lower case
        input.pop(); // remove the final character (newline)
        process_input(&input);
        input.clear();
    }
}
