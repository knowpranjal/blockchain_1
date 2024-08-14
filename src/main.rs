mod block;
mod blockchain;
mod genesis;

use crate::blockchain::Blockchain;

fn main() {
    let mut blockchain = Blockchain::new();

    blockchain.add_block("First block after Genesis".to_string());
    blockchain.add_block("Second block after Genesis".to_string());

    println!("Blockchain: {:?}", blockchain);

    let is_valid = blockchain.is_valid();
    println!("Is blockchain valid? {}", is_valid);
}
