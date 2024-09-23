use std::collections::{HashMap, HashSet};

/// Represents a transaction to be included in a block.
#[derive(Debug, Clone)]
pub struct BlockTransaction {
    pub id: u64,
    pub sender: String,
    pub receiver: String,
    pub amount: u64,
}

impl BlockTransaction {
    pub fn new(sender: String, receiver: String, amount: u64) -> Self {
        // Generate unique ID for each transaction
        static mut TRANSACTION_COUNTER: u64 = 1;
        let id;
        unsafe {
            id = TRANSACTION_COUNTER;
            TRANSACTION_COUNTER += 1;
        }
        BlockTransaction {
            id,
            sender,
            receiver,
            amount,
        }
    }
}

/// Represents a block in the blockchain DAG.
#[derive(Debug, Clone)]
pub struct Block {
    pub id: String,
    pub transactions: Vec<BlockTransaction>,
    pub parent_ids: Vec<String>,
    pub child_ids: Vec<String>,
}

/// Represents the blockchain as a DAG.
#[derive(Debug)]
pub struct DAG {
    pub blocks: HashMap<String, Block>, // Stores all blocks by their ID.
    pub tips: HashSet<String>,          // Blocks without children (the tips of the DAG).
    pub current_height: u64,            // Current height of the blockchain.
}

impl DAG {
    /// Initializes a new blockchain with a genesis block.
    pub fn new() -> Self {
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
        }
    }

    /// Adds transactions to the blockchain, handling splitting logic.
    pub fn add_transactions(&mut self, transactions: Vec<BlockTransaction>) -> Result<(), String> {
        if transactions.is_empty() {
            return Ok(()); // No transactions to add.
        }

        // Collect current tips as parents for the new block(s).
        let parent_ids: Vec<String> = self.tips.iter().cloned().collect();
        self.tips.clear(); // Clear tips since we'll add new blocks.

        // Increment the current height.
        self.current_height += 1;
        let height = self.current_height;

        // Calculate the number of blocks needed.
        let max_transactions_per_block = 5;
        let num_blocks = (transactions.len() + max_transactions_per_block - 1) / max_transactions_per_block;

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
            println!(
                "  Transactions: {:?}",
                block.transactions.iter().map(|tx| (tx.id, &tx.sender, &tx.receiver, tx.amount)).collect::<Vec<_>>()
            );
            println!("");
        }
    }
}
