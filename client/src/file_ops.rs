use sha3::{Digest, Sha3_512};
use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::{BufRead, Read, Seek, Write};

use crate::config::{self};

/**
 * Hashes a given file using Sha3_512
 * NOTE: This function is VERY slow when project is built for debug, use release for faster hashing or use small files in testing
 * @param path: String - The path to the file to hash
 * @return String - The hash of the file
 */
pub fn hash_file(path: &str) -> Result<String, std::io::Error> {
    let mut file = File::open(path)?;
    let mut buffer = vec![0u8; 16 * 1024 * 1024];
    let mut hasher = Sha3_512::new();

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    return Ok(format!("{:x}", result));
}

/**
 * Hash the given buffer, used to generate a torrent file
 */
pub fn hash_buffer(buffer: &Vec<u8>) -> String {
    let mut hasher = Sha3_512::new();
    hasher.update(buffer);
    let result = hasher.finalize();
    return format!("{:x}", result);
}

/**
 * Hash files in a directory at a shallow level, return a hashmap of the file path to it's corresponding hash
 * @param path: &str - The path to the directory
 * @return HashMap<String, String> - A hashmap of the file path to it's corresponding hash
 */
pub fn hash_files_shallow(path: &str) -> Result<HashMap<String, String>, std::io::Error> {
    let mut files_map: HashMap<String, String> = HashMap::new();
    let (_, files) = get_directory_children(path)?;
    for f in files {
        files_map.insert(f.clone(), hash_file(&f)?);
    }

    return Ok(files_map);
}

/**
 * Gets the children of a directory
 * @param path: &str - The path to the directory
 * @return (Vec<String>, Vec<String>) - A tuple containing the directories and files in the directory
 */
pub fn get_directory_children(path: &str) -> Result<(Vec<String>, Vec<String>), std::io::Error> {
    let mut dirs: Vec<String> = Vec::new();
    let mut files: Vec<String> = Vec::new();

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        let path_str = match path.to_str() {
            Some(result) => result,
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to convert path to string",
                ));
            }
        };
        if path.is_dir() {
            dirs.push(String::from(path_str));
        } else {
            files.push(String::from(path_str));
        }
    }

    return Ok((dirs, files));
}

/**
 * Receive a specific block from a file
 * @return Result resolving to a buffer
 */
pub fn get_block(file_path: String, block_num: u64) -> Result<Vec<u8>, std::io::Error> {
    let mut file = File::open(file_path)?;
    let file_size = file.metadata()?.len();
    let is_final_block: bool = ((block_num + 1) * config::BLOCK_SIZE) > file_size
        || ((block_num + 1) * config::BLOCK_SIZE) == file_size;
    // Seek to starting byte
    file.seek(std::io::SeekFrom::Start(block_num * config::BLOCK_SIZE))?;

    let mut buffer: Vec<u8>;
    match is_final_block {
        true => {
            buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
        }
        false => {
            buffer = vec![0u8; config::BLOCK_SIZE as usize];
            file.read_exact(&mut buffer)?;
        }
    }
    return Ok(buffer);
}

/**
 * Generate an RDFS file mainly made up of primary hash, block size, as well as the number and hash of each block
 */
pub fn generate_rdfs_file(input_path: &str) -> Result<String, std::io::Error> {
    // Get some initial data from the file's metadata
    let path = std::path::Path::new(input_path);
    let mut input_file = File::open(path)?;
    let file_size = input_file.metadata()?.len();
    let full_blocks_num = file_size / config::BLOCK_SIZE;
    let final_block_size = file_size % config::BLOCK_SIZE;
    let file_name = path
        .file_name()
        .ok_or(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Could not find file name",
        ))?
        .to_str()
        .ok_or(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Could not convert file name to string",
        ))?;
    let output_path = format!("./generated/{}.rdfs", file_name);

    // Open a write stream to the output file
    let mut output_file = File::create(output_path.clone())?;

    // Write the first line of the torrent file
    let file_hash = hash_file(input_path)?;
    let line_1 = format!("#S {} {file_size} {file_hash}\n", config::BLOCK_SIZE);
    output_file.write(line_1.as_bytes())?;

    // Create a buffer of size BLOCK_SIZE
    let mut buffer = vec![0u8; config::BLOCK_SIZE as usize];

    // Write Our Completed Block Hashes
    for _ in 0..full_blocks_num {
        input_file.read_exact(&mut buffer)?;
        let next_line = format!("{}\n", hash_buffer(&buffer));
        output_file.write(next_line.as_bytes())?;
    }

    // Write the size + hash value of the final block, will be 0 if data is split even across the block size
    buffer = vec![0u8; final_block_size as usize];
    input_file.read_exact(&mut buffer)?;
    let final_line = format!("#E {final_block_size} {}\n", hash_buffer(&buffer));
    output_file.write(final_line.as_bytes())?;

    // Ensure entire file has been read
    if input_file.read(&mut buffer)? != 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "Not all data was read from the file",
        ));
    }

    return Ok(output_path);
}

/**
 * Zero out a file at a given path with the specified size
 * @param path: &str - The path to the file
 * @param size: u64 - The size of the file to create
 * @return Result<(), std::io::Error> - Result indicating success or failure
 */
pub fn reserve_file_space(path: &str, size: u64) -> Result<(), std::io::Error> {
    let file = File::create(path)?;
    file.set_len(size)?;
    Ok(())
}

/**
 * Write a block to a file at a given path
 * @param path: &str - The path to the file
 * @param block_num: u64 - The block number to write
 * @param data: &[u8] - The data to write
 * @return Result<(), std::io::Error> - Result indicating success or failure
 */
pub fn write_block(path: &str, block_num: u64, data: Vec<u8>) -> Result<(), std::io::Error> {
    let mut file = File::options().write(true).open(path)?;
    file.seek(std::io::SeekFrom::Start(block_num * config::BLOCK_SIZE))?;
    file.write_all(&data)?;
    Ok(())
}

/**
 * Return vector of lines from the file path provided
 * @return Vec<String> - Vector of lines from the file
 */
pub fn get_raw_lines(file_path: &str) -> Result<Vec<String>, std::io::Error> {
    let mut lines: VecDeque<String> = VecDeque::new();
    let file = File::open(file_path)?;
    let mut reader = std::io::BufReader::new(file);
    let mut line = String::new();
    let mut result = reader.read_line(&mut line)?;
    while result > 0 {
        lines.push_front(line.clone());
        line.clear();
        result = reader.read_line(&mut line)?;
    }
    return Ok(Vec::from(lines));
}
