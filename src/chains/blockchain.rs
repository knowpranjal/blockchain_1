use crate::core::block::Block;

pub struct Blockchain {
    pub blocks: Vec<Block>,
}

impl Blockchain {
    pub fn new() -> Blockchain {
        let genesis_block = Block::new(0, String::from("0"), String::from("Genesis Block"));
        Blockchain {
            blocks: vec![genesis_block],
        }
    }

    pub fn add_block(&mut self, data: String) {
        let previous_block = self.blocks.last().unwrap().clone();
        let new_block = Block::new(previous_block.index + 1, previous_block.hash, data);
        self.blocks.push(new_block);
    }

    pub fn _is_valid(&self) -> bool {
        for i in 1..self.blocks.len() {
            let current_block = &self.blocks[i];
            let previous_block = &self.blocks[i - 1];

            if current_block.hash != Block::calculate_hash(
                current_block.index,
                current_block.timestamp,
                &current_block.previous_hash,
                &current_block.data,
            ) {
                return false;
            }

            if current_block.previous_hash != previous_block.hash {
                return false;
            }
        }
        true
    }
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blockchain_creation() {
        let blockchain = Blockchain::new();

        assert_eq!(blockchain.blocks.len(), 1); // Genesis block should be the only block
        assert_eq!(blockchain.blocks[0].data, "Genesis Block");
        assert_eq!(blockchain.blocks[0].index, 0);
    }

    #[test]
    fn test_add_block() {
        let mut blockchain = Blockchain::new();
        blockchain.add_block(String::from("First block"));
        blockchain.add_block(String::from("Second block"));

        assert_eq!(blockchain.blocks.len(), 3); // Genesis + 2 blocks
        assert_eq!(blockchain.blocks[1].data, "First block");
        assert_eq!(blockchain.blocks[2].data, "Second block");

        // Check if the indices are correct
        assert_eq!(blockchain.blocks[1].index, 1);
        assert_eq!(blockchain.blocks[2].index, 2);

        // Check if the previous hash of the second block matches the hash of the first block
        assert_eq!(blockchain.blocks[2].previous_hash, blockchain.blocks[1].hash);
    }

    #[test]
    fn test_blockchain_validity() {
        let mut blockchain = Blockchain::new();
        blockchain.add_block(String::from("First block"));
        blockchain.add_block(String::from("Second block"));

        assert!(blockchain._is_valid()); // Blockchain should be valid
    }

    #[test]
    fn test_blockchain_invalidity_due_to_modified_data() {
        let mut blockchain = Blockchain::new();
        blockchain.add_block(String::from("First block"));
        blockchain.add_block(String::from("Second block"));

        // Manually modify a block's data
        blockchain.blocks[1].data = String::from("Tampered block");

        assert!(!blockchain._is_valid()); // Blockchain should be invalid
    }

    #[test]
    fn test_blockchain_invalidity_due_to_modified_hash() {
        let mut blockchain = Blockchain::new();
        blockchain.add_block(String::from("First block"));
        blockchain.add_block(String::from("Second block"));

        // Manually modify a block's hash
        blockchain.blocks[1].hash = String::from("1234567890abcdef");

        assert!(!blockchain._is_valid()); // Blockchain should be invalid
    }

    #[test]
    fn test_blockchain_invalidity_due_to_modified_previous_hash() {
        let mut blockchain = Blockchain::new();
        blockchain.add_block(String::from("First block"));
        blockchain.add_block(String::from("Second block"));

        // Manually modify the previous hash of the second block
        blockchain.blocks[2].previous_hash = String::from("abcdef1234567890");

        assert!(!blockchain._is_valid()); // Blockchain should be invalid
    }
}

