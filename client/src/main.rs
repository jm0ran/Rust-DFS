use std::net::TcpStream;
use std::io::Write;

mod file_ops;
use file_ops::hash_file;

const TARGET: &str = "127.0.0.1:7777";

/**
 * What does a message to the listener server look like?:
 * RDFS 0.1 DISCOVERY_POST
 * #D
 * Hash1
 * Hash2
 * Hash3
 * ...
 * #R
 * Hash1
 * Hash2
 * Hash3
 * ...
 */
fn construct_request(distributing: &Vec<String>, requesting: &Vec<String>) -> Vec<String>{
    let mut response: Vec<String> = Vec::new();
    response.push(String::from("#S RDFT 0.1 DISCOVERY_POST\n"));
  
    // Add all distributing files to the request
    response.push(String::from("#D\n"));
    for s in distributing {
        response.push(format!("{}\n", s.clone()));
    }
    
    // Add all requesting files to the request
    response.push(String::from("#R\n"));
    for s in requesting {
        response.push(format!("{}\n", s.clone()));
    }

    // Indicate end of transmission
    response.push(String::from("#E\n"));

    return response;
}

fn main() {    
    // // Create a temporary distributing array
    // let mut distributing: Vec<String> = Vec::new();
    // distributing.push(hash_file("temp/temp.mp4"));
    // distributing.push(String::from("D2"));

    // // Create a temporary requesting array
    // let mut requesting: Vec<String> = Vec::new();
    // requesting.push(String::from("R1"));
    // requesting.push(String::from("R2"));

    // // Construct the request
    // let request = construct_request(&distributing, &requesting);

    // // Open the stream and send the request
    // let mut stream: TcpStream = TcpStream::connect(TARGET).unwrap();
    // for line in request{
    //     stream.write_all(line.as_bytes()).unwrap();
    // }

    // Test directory scanning
    let (dirs, files) = file_ops::get_directory_children("temp");
    for d in dirs{
        println!("Dir: {:x?}", d);
    }
    for f in files{
        println!("File: {:x?}", f);
    }   
}
