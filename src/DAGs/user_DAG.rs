use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: String,
    pub sender: String,
    pub receiver: String,
    pub amount: u64,
    pub parent_id: Option<String>,    // Parent transaction ID
    pub child_ids: Vec<String>,       // Children transaction IDs
}

impl Transaction {
    pub fn new(sender: String, receiver: String, amount: u64, parent_id: Option<String>) -> Self {
        Transaction {
            id: Uuid::new_v4().to_string(), // Generate unique ID for each transaction
            sender,
            receiver,
            amount,
            parent_id,
            child_ids: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LocalDAG {
    pub transactions: HashMap<String, Transaction>,
    pub latest_transaction_id: Option<String>, // Keep track of the latest transaction
}

impl LocalDAG {
    pub fn new() -> Self {
        LocalDAG {
            transactions: HashMap::new(),
            latest_transaction_id: None,
        }
    }

    pub fn add_transaction(&mut self, sender: String, receiver: String, amount: u64) -> Result<(), String> {
        let parent_id = self.latest_transaction_id.clone(); // The parent is the latest transaction
        let transaction = Transaction::new(sender.clone(), receiver.clone(), amount, parent_id.clone());

        // If there was a previous transaction, update it to link to this new one as a child
        if let Some(parent_id) = parent_id {
            if let Some(parent_transaction) = self.transactions.get_mut(&parent_id) {
                parent_transaction.child_ids.push(transaction.id.clone());
            }
        }

        // Add the new transaction to the local DAG
        self.transactions.insert(transaction.id.clone(), transaction.clone());

        // Update the latest transaction ID to the new transaction
        self.latest_transaction_id = Some(transaction.id.clone());

        Ok(())
    }

    pub fn print_dag(&self) {
        println!("Printing DAG for the user:");
        for (id, transaction) in &self.transactions {
            println!(
                "Transaction ID: {}, Sender: {}, Receiver: {}, Amount: {}, Parent: {:?}, Children: {:?}",
                transaction.id, 
                transaction.sender, 
                transaction.receiver, 
                transaction.amount, 
                transaction.parent_id, 
                transaction.child_ids
            );
        }
    }

    pub fn get_transaction(&self, transaction_id: &str) -> Option<&Transaction> {
        self.transactions.get(transaction_id)
    }
}

