use crate::block::Block;

pub fn create_genesis_block(index: u32, data: String, hash: String) -> Block {
    Block::new(
        index,
        data,
        hash,
    )
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_block_creation() {
    let genesis_block = create_genesis_block(0, String::from("Genesis block"), String::from("000000000"));
    assert_eq!(genesis_block.index, 0);
    assert_eq!(genesis_block.data, String::from("Genesis block"));
    assert_eq!(genesis_block.previous_hash, String::from("000000000"));
    }

}