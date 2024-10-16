// transaction_dag.rs

use std::collections::{HashMap, HashSet};
use std::sync::{Arc};
use tokio::sync::Mutex;
use crate::models::user::UserPool;
use crate::models::pki::KeyPairWrapper;
use serde::{Serialize, Deserialize};
use futures::executor::block_on;


/// Represents a transaction to be included in a block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockTransaction {
    pub id: String, // Change from u64 to String
    pub sender: String,
    pub receiver: String,
    pub amount: u64,
    pub signature: Vec<u8>, // Signature
    pub timestamp: u64,     // Timestamp
}

impl BlockTransaction {
    pub fn new(
        id: String, // Use the provided transaction ID
        sender: String,
        receiver: String,
        amount: u64,
        signature: Vec<u8>,
        timestamp: u64,
    ) -> Self {
        BlockTransaction {
            id,
            sender,
            receiver,
            amount,
            signature,
            timestamp,
        }
    }
}

/// Represents a block in the blockchain DAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub id: String,
    pub transactions: Vec<BlockTransaction>,
    pub parent_ids: Vec<String>,
    pub child_ids: Vec<String>,
}

/// Represents the blockchain as a DAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = ""))]
pub struct DAG {
    pub blocks: HashMap<String, Block>, // Stores all blocks by their ID.
    pub tips: HashSet<String>,          // Blocks without children (the tips of the DAG).
    pub current_height: u64,            // Current height of the blockchain.
    #[serde(skip_serializing, skip_deserializing)]
    pub user_pool: Arc<Mutex<UserPool>>, // Reference to the user pool
}

impl DAG {
    /// Initializes a new blockchain with a genesis block.
    pub fn new(user_pool: Arc<Mutex<UserPool>>) -> Self {
        let genesis_block = Block {
            id: "1".to_string(),
            transactions: Vec::new(),
            parent_ids: Vec::new(),
            child_ids: Vec::new(),
        };

        let mut blocks = HashMap::new();
        blocks.insert("1".to_string(), genesis_block);

        let mut tips = HashSet::new();
        tips.insert("1".to_string());

        DAG {
            blocks,
            tips,
            current_height: 1,
            user_pool,
        }
    }

    pub fn get_transaction(&self, transaction_id: &str) -> Option<BlockTransaction> {
        for block in self.blocks.values() {
            for tx in &block.transactions {
                if tx.id == transaction_id {
                    return Some(tx.clone());
                }
            }
        }
        None
    }

    /// Retrieves a user's public key
    async fn get_user_public_key(&self, username: &str) -> Option<Vec<u8>> {
        let pool = self.user_pool.lock().await;
        pool.get_user_public_key(username)
    }

    /// Adds transactions to the blockchain, handling splitting logic.
    pub async fn add_transactions(&mut self, transactions: Vec<BlockTransaction>) -> Result<(), String> {
        if transactions.is_empty() {
            return Ok(()); // No transactions to add.
        }

        // Verify signatures of all transactions
        for tx in &transactions {
            // Reconstruct the message, including the transaction ID
            let message = format!(
                "{}:{}:{}:{}:{}",
                tx.id, tx.sender, tx.receiver, tx.amount, tx.timestamp
            );

            // Retrieve the sender's public key
            // Note: Since `get_user_public_key` is async, we need to adjust accordingly.
            let sender_public_key = futures::executor::block_on(self.get_user_public_key(&tx.sender))
                .ok_or(format!("Sender public key not found for {}", tx.sender))?;

            // Verify the signature
            KeyPairWrapper::verify(
                &sender_public_key,
                message.as_bytes(),
                &tx.signature,
            )
            .map_err(|e| format!("Signature verification failed for transaction {}: {:?}", tx.id, e))?;
        }

        // Collect current tips as parents for the new block(s).
        let parent_ids: Vec<String> = self.tips.iter().cloned().collect();
        self.tips.clear(); // Clear tips since we'll add new blocks.

        // Increment the current height.
        self.current_height += 1;
        let height = self.current_height;

        // Calculate the number of blocks needed.
        let max_transactions_per_block = 5;
        let num_blocks =
            (transactions.len() + max_transactions_per_block - 1) / max_transactions_per_block;

        // Split transactions into chunks of up to max_transactions_per_block.
        let mut new_block_ids = Vec::new();
        for i in 0..num_blocks {
            let start_index = i * max_transactions_per_block;
            let end_index = usize::min(start_index + max_transactions_per_block, transactions.len());
            let txns = transactions[start_index..end_index].to_vec();

            // Generate block ID.
            let block_id = if num_blocks == 1 {
                // If only one block at this height, use height as ID.
                height.to_string()
            } else {
                // Multiple blocks, use format "height.index".
                format!("{}.{}", height, i + 1)
            };

            let block = Block {
                id: block_id.clone(),
                transactions: txns,
                parent_ids: parent_ids.clone(),
                child_ids: Vec::new(),
            };

            self.blocks.insert(block_id.clone(), block);
            self.tips.insert(block_id.clone());
            new_block_ids.push(block_id.clone());
        }

        // Update parent blocks to include all new blocks as their children.
        for parent_id in &parent_ids {
            if let Some(parent_block) = self.blocks.get_mut(parent_id) {
                parent_block.child_ids.extend(new_block_ids.clone());
            }
        }

        Ok(())
    }

    /// Prints the entire blockchain for debugging purposes.
    pub fn print_chain(&self) {
        println!("Blockchain DAG:");
        for block in self.blocks.values() {
            println!("Block ID: {}", block.id);
            println!("  Parents: {:?}", block.parent_ids);
            println!("  Children: {:?}", block.child_ids);
            println!("  Transactions:");
            for tx in &block.transactions {
                println!(
                    "    Transaction ID: {}, Sender: {}, Receiver: {}, Amount: {}, Timestamp: {}, Signature: {:?}",
                    tx.id, tx.sender, tx.receiver, tx.amount, tx.timestamp, tx.signature
                );
            }
            println!("");
        }
    }
}
