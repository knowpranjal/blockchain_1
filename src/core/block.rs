use std::time::{SystemTime, UNIX_EPOCH};
use sha2::{Sha256, Digest};

#[derive(Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: u128,
    pub previous_hash: String,
    pub hash: String,
    pub data: String,
}

impl Block {
    pub fn new(index: u64, previous_hash: String, data: String) -> Block {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();
        let hash = Block::calculate_hash(index, timestamp, &previous_hash, &data);

        Block {
            index,
            timestamp,
            previous_hash,
            hash,
            data,
        }
    }

    pub fn calculate_hash(index: u64, timestamp: u128, previous_hash: &str, data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(index.to_string());
        hasher.update(timestamp.to_string());
        hasher.update(previous_hash);
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }
}





#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_creation() {
        let data = String::from("Block data");
        let previous_hash = String::from("00000000000000000000000000000000");
        let block = Block::new(1, previous_hash.clone(), data.clone());

        assert_eq!(block.index, 1);
        assert_eq!(block.previous_hash, previous_hash);
        assert_eq!(block.data, data);
        assert!(!block.hash.is_empty());
    }

    #[test]
    fn test_hash_calculation() {
        let index = 1;
        let timestamp = 1630348284_u128;
        let previous_hash = "00000000000000000000000000000000";
        let data = "Block data";
        let expected_hash = Block::calculate_hash(index, timestamp, previous_hash, data);

        // Manually calculate the hash to check correctness
        let mut hasher = Sha256::new();
        hasher.update(index.to_string());
        hasher.update(timestamp.to_string());
        hasher.update(previous_hash);
        hasher.update(data);
        let manual_hash = format!("{:x}", hasher.finalize());

        assert_eq!(expected_hash, manual_hash);
    }

    #[test]
    fn test_different_data_produces_different_hashes() {
        let previous_hash = "00000000000000000000000000000000".to_string();
        let block1 = Block::new(1, previous_hash.clone(), "Data 1".to_string());
        let block2 = Block::new(2, previous_hash.clone(), "Data 2".to_string());

        assert_ne!(block1.hash, block2.hash);
    }

    #[test]
    fn test_different_timestamps_produce_different_hashes() {
        let previous_hash = "00000000000000000000000000000000".to_string();
        let block1 = Block::new(1, previous_hash.clone(), "Same data".to_string());
        
        // Wait for 1 millisecond to ensure different timestamps
        std::thread::sleep(std::time::Duration::from_millis(1));
        
        let block2 = Block::new(1, previous_hash.clone(), "Same data".to_string());

        assert_ne!(block1.hash, block2.hash);
    }

    #[test]
    fn test_same_data_same_hash() {
        let previous_hash = "00000000000000000000000000000000".to_string();
        let block1 = Block::new(1, previous_hash.clone(), "Same data".to_string());
        let block2 = Block::new(1, previous_hash.clone(), "Same data".to_string());

        assert_eq!(block1.hash, block2.hash);
    }
}

