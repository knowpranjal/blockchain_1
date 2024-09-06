use crate::block::Block;

pub struct MainChain {
    pub blocks: Vec<Block>,
}

impl MainChain {
    pub fn new() -> MainChain {
        // Create a genesis block for the main chain
        let genesis_block = Block::new(0, String::from("0"), String::from("Genesis Block"));
        MainChain {
            blocks: vec![genesis_block],
        }
    }

    pub fn add_transaction_hash(&mut self, transaction_hash: String) {
        // Get the previous block and use its hash for the new block
        let previous_block = self.blocks.last().unwrap().clone();
        let new_block = Block::new(previous_block.index + 1, previous_block.hash, transaction_hash);
        self.blocks.push(new_block);
    }

    pub fn is_valid(&self) -> bool {
        // Check the integrity of the main chain
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
    fn test_mainchain_creation() {
        let main_chain = MainChain::new();
        assert_eq!(main_chain.blocks.len(), 1); // Genesis block should be the only block
        assert_eq!(main_chain.blocks[0].data, "Genesis Block");
        assert_eq!(main_chain.blocks[0].index, 0);
    }

    #[test]
    fn test_add_transaction_hash() {
        let mut main_chain = MainChain::new();
        main_chain.add_transaction_hash(String::from("hash_of_tx1"));
        main_chain.add_transaction_hash(String::from("hash_of_tx2"));

        assert_eq!(main_chain.blocks.len(), 3); // Genesis + 2 blocks
        assert_eq!(main_chain.blocks[1].data, "hash_of_tx1");
        assert_eq!(main_chain.blocks[2].data, "hash_of_tx2");

        // Check if the indices are correct
        assert_eq!(main_chain.blocks[1].index, 1);
        assert_eq!(main_chain.blocks[2].index, 2);

        // Check if the previous hash of the second block matches the hash of the first block
        assert_eq!(main_chain.blocks[2].previous_hash, main_chain.blocks[1].hash);
    }

    #[test]
    fn test_mainchain_validity() {
        let mut main_chain = MainChain::new();
        main_chain.add_transaction_hash(String::from("hash_of_tx1"));
        main_chain.add_transaction_hash(String::from("hash_of_tx2"));

        assert!(main_chain.is_valid()); // Main chain should be valid
    }

    #[test]
    fn test_mainchain_invalidity_due_to_modified_data() {
        let mut main_chain = MainChain::new();
        main_chain.add_transaction_hash(String::from("hash_of_tx1"));
        main_chain.add_transaction_hash(String::from("hash_of_tx2"));

        // Manually modify a block's data
        main_chain.blocks[1].data = String::from("Tampered hash");

        assert!(!main_chain.is_valid()); // Main chain should be invalid
    }
}
