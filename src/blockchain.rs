use crate::block::Block;
use crate::genesis::create_genesis_block;

#[derive(Debug)]
pub struct Blockchain {
    pub chain: Vec<Block>,
}

impl Blockchain {
    // Create a new blockchain with the genesis block
    pub fn new() -> Blockchain {
        let mut blockchain = Blockchain { chain: Vec::new() };
        blockchain.add_genesis_block();
        blockchain
    }

    // Add the genesis block to the blockchain
    fn add_genesis_block(&mut self) {
        let genesis_block = create_genesis_block(0, "Genesis block".to_string(), "000000000".to_string());
        self.chain.push(genesis_block);
    }

    // Add a new block to the blockchain
    pub fn add_block(&mut self, data: String) {
        let previous_hash = self.chain.last().unwrap().hash.clone();
        let new_block = Block::new(self.chain.len() as u32, data, previous_hash);
        self.chain.push(new_block);
    }

    // Validate the blockchain's integrity
    pub fn is_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];

            // Validate the hash
            if current_block.hash != current_block.calculate_hash() {
                return false;
            }

            // Validate the linkage
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

    //adding a new block
    #[test]
    fn test_add_block() {
    let mut blockchain = Blockchain::new();
    let data = String::from("Block data");

    blockchain.add_block(data.clone());

    let latest_block = blockchain.chain.last().unwrap();
    assert_eq!(latest_block.data, data);
    assert_ne!(latest_block.hash, "".to_string()); // Ensure hash is not empty
    }

    //validating the blockchain
    #[test]
    fn test_valid_chain() {
    let mut blockchain = Blockchain::new();
    let data1 = String::from("Block 1 data");
    let data2 = String::from("Block 2 data");

    blockchain.add_block(data1.clone());
    blockchain.add_block(data2.clone());

    assert!(blockchain.is_valid());
    }

    //validating hamperred chain
    #[test]
    fn test_invalid_chain_tampered_hash() {
    let mut blockchain = Blockchain::new();
    let data1 = String::from("Block 1 data");
    let data2 = String::from("Block 2 data");

    blockchain.add_block(data1.clone());
    blockchain.add_block(data2.clone());

    // Tamper with the hash of the second block
    let second_block = blockchain.chain.get_mut(1).unwrap();
    second_block.hash = String::from("tampered_hash");

    assert!(!blockchain.is_valid());
    }

}