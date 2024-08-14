use sha2::{Sha256, Digest};                             //library for hashing
use std::time::{SystemTime, UNIX_EPOCH};                //calculate time for each block

#[derive(Debug)]
//a structure to store all the block's property
pub struct Block {
    pub index: u32,
    pub timestamp: u128,
    pub data: String,
    pub hash: String,
    pub previous_hash: String,
}


//implementation of properties in block struct and creation of new blocks
impl Block {
    //function to produce new blocks
    pub fn new(index: u32, data: String, previous_hash: String) -> Block {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();
        let mut block = Block {
            index,
            timestamp,
            data,
            previous_hash: previous_hash.clone(),
            hash: String::new(),
        };
        block.hash = block.calculate_hash();
        block
    }


    //calculating hash for the new block on the basis of its overall data
    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!(
            "{}{}{}{}",
            self.index, self.timestamp, self.data, self.previous_hash
        ));
        format!("{:x}", hasher.finalize())
    }
}




//test cases to verify the functionality of the block, will only run on cargo test command
#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Sha256, Digest};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_block_creation() {
        let index = 1;
        let data = String::from("Test data");
        let previous_hash = String::from("00000000000000000000000000000");

        let block = Block::new(index, data.clone(), previous_hash.clone());

        assert_eq!(block.index, index);
        assert_eq!(block.data, data);
        assert_eq!(block.previous_hash, previous_hash);
        assert_ne!(block.hash, ""); // Hash should not be empty
    }

    #[test]
    fn test_block_hash_calculation() {
        let index = 1;
        let data = String::from("Test data");
        let previous_hash = String::from("00000000000000000000000000000");
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();

        let block = Block::new(index, data.clone(), previous_hash.clone());

        let mut hasher = Sha256::new();
        hasher.update(format!("{}{}{}{}", index, timestamp, data, previous_hash));
        let expected_hash = format!("{:x}", hasher.finalize());

        assert_eq!(block.hash, expected_hash);
    }

    #[test]
    fn test_blockchain_integrity() {
        let first_block = Block::new(0, String::from("Genesis block"), String::from("00000000000000000000000000000"));
        let second_block = Block::new(1, String::from("Second block"), first_block.hash.clone());

        assert_eq!(second_block.previous_hash, first_block.hash);
        assert_ne!(second_block.hash, first_block.hash); // Ensure hashes differ
    }
}
