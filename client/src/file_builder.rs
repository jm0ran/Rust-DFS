/**
 * File builder class is in charge of building each individual file sending requests for blocks of data and then putting these blocks together
 */

enum FileStatus {
    AwaitingDistributor,
    AwaitingSize,
    InProgress,
    Complete,
}

enum BlockState {
    Waiting,
    InProgress,
    Complete,
}
pub struct FileBuilder {
    size: Option<i64>, // Size of target file, will need to request this from the other client, will not be set until size of file is known
    blocks_complete: Option<Vec<BlockState>>, // Blocks complete, an array of BlockState enums to track status of each block, will not be created until size of file is known
    currently_downloading: i32, // Number of blocks currently downloading, should not exceed a set amount, not sure if this will be enforced per file or across the entire system
}

impl FileBuilder {
    pub fn new() -> FileBuilder {
        FileBuilder {
            size: None,
            blocks_complete: None,
            currently_downloading: 0,
        }
    }
}
