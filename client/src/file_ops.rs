use sha3::{Digest, Sha3_512};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

/**
 * Hashes a given file using Sha3_512
 * NOTE: This function is VERY slow when project is built for debug, use release for faster hashing or use small files in testing
 * @param path: String - The path to the file to hash
 * @return String - The hash of the file
 */
pub fn hash_file(path: &str) -> String {
    let mut file = File::open(path).expect("Unable to open file");
    let mut buffer = vec![0u8; 2 * 1024 * 1024];
    let mut hasher = Sha3_512::new();

    loop {
        let bytes_read = file.read(&mut buffer).expect("Unable to read file");
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    return format!("{:x}", result);
}

/**
 * Hash files in a directory at a shallow level, return a hashmap of the file path to it's corresponding hash
 * @param path: &str - The path to the directory
 * @return HashMap<String, String> - A hashmap of the file path to it's corresponding hash
 */
pub fn hash_files_shallow(path: &str) -> HashMap<String, String> {
    let mut files_map: HashMap<String, String> = HashMap::new();
    let (_, files) = get_directory_children(path);
    for f in files {
        files_map.insert(f.clone(), hash_file(&f));
    }

    return files_map;
}

/**
 * Gets the children of a directory
 * @param path: &str - The path to the directory
 * @return (Vec<String>, Vec<String>) - A tuple containing the directories and files in the directory
 */
pub fn get_directory_children(path: &str) -> (Vec<String>, Vec<String>) {
    let mut dirs: Vec<String> = Vec::new();
    let mut files: Vec<String> = Vec::new();

    for entry in std::fs::read_dir(path).expect("Unable to read directory") {
        let entry = entry.expect("Unable to read entry");
        let path = entry.path();
        let path_str = path.to_str().expect("Unable to convert path to string");
        if path.is_dir() {
            dirs.push(String::from(path_str));
        } else {
            files.push(String::from(path_str));
        }
    }

    return (dirs, files);
}
