/**
 * Plain Text User Interface For The Client
 */
use std::io::{self, Write};
use std::str::Split;

pub mod config;
pub mod file_builder;
pub mod file_ops;
pub mod linker_comm;

mod file_manager;
mod linker;

fn linker_cmd(mut line_parts: Split<'_, &str>) {
    // Get reference to the linker singleton
    let linker = linker::Linker::instance();

    // Ensure line has a next argument
    let arg_1;
    match line_parts.next() {
        Some(arg) => {
            arg_1 = arg;
        }
        None => {
            println!("Did not provide expect arguments for linker command");
            return;
        }
    }

    match arg_1 {
        "set" => {
            let new_target: String;
            match line_parts.next() {
                Some(target) => {
                    new_target = String::from(target); // Will want some input validation here eventually
                }
                None => {
                    println!("Did not provide a new linking server address to set");
                    return;
                }
            }
            {
                let mut lock = linker.write().unwrap();
                lock.set_target(new_target);
            }
        }
        "get" => {
            let target: String;
            {
                let lock = linker.read().unwrap();
                target = lock.get_target();
            }
            println!("Target: {target}")
        }
        "update" => {
            let mut lock = linker.write().unwrap();
            match lock.update() {
                Some(err) => {
                    println!("Linker Update Encountered an Error: {}", err);
                }
                None => {
                    println!("Linker Updated Successfully"); // TODO: Determine if I even want to print for positive output
                }
            }
        }
        "debug" => {
            let lock = linker.read().unwrap();
            lock.debug();
        }
        _ => {
            println!("Unrecognized Linker Argument: {}", arg_1);
        }
    }
}

fn file_cmd(mut line_parts: Split<'_, &str>) {
    // Get reference to the file manager singleton
    let file_manager = file_manager::FileManager::instance();

    // Ensure line has a next argument
    let arg_1;
    match line_parts.next() {
        Some(arg) => {
            arg_1 = arg;
        }
        None => {
            println!("Did not provide expect arguments for files command");
            return;
        }
    }

    match arg_1 {
        "scan" => {
            let mut lock = file_manager.write().unwrap();
            match lock.scan_distributing() {
                Some(err) => {
                    println!(
                        "Encountered an error when scanning the distribution directory: {err}"
                    );
                }
                None => {
                    println!("Scan Complete");
                }
            }
        }
        "distributing" => {
            let lock = file_manager.read().unwrap();
            let distributing = lock.get_distributing();
            println!("Files For Distribution");
            for entry in distributing.iter() {
                println!("\t{} : {}", entry.0, entry.1);
            }
        }
        _ => {
            println!("Unrecognized Files Argument: {}", arg_1);
        }
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
        }
        "clear" => {
            print!("\x1B[2J\x1B[1;1H"); // Clear terminal screen
            io::stdout().flush().unwrap();
        }
        "linker" => {
            linker_cmd(line_parts);
        }
        "files" => {
            file_cmd(line_parts);
        }
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

fn main() {
    {
        // Temporarily register some requesting hashes because I'm lazy
        let file_manager = file_manager::FileManager::instance();
        let mut lock = file_manager.write().unwrap();
        lock.register_requesting((String::from("./receiving/req-1.txt"), String::from("2dd184b8c84b999a6ccc7ae4da2efc3b3cd455d50a04686caaf90f8f5cd60194c8e0e758947738f1001e01010ddb28e782ed274c966561ba798fe0123f495b5d")));
    }
    run();
}
