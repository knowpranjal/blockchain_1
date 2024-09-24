use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: String,
    pub sender: String,
    pub receiver: String,
    pub amount: u64,
    pub parent_id: Option<String>, // Parent transaction ID
    pub child_ids: Vec<String>,    // Children transaction IDs
    pub signature: Vec<u8>,        // New field: Signature
    pub timestamp: u64,            // New field: Timestamp
}

impl Transaction {
    pub fn new(
        sender: String,
        receiver: String,
        amount: u64,
        parent_id: Option<String>,
        signature: Vec<u8>,
        timestamp: u64,
    ) -> Self {
        Transaction {
            id: Uuid::new_v4().to_string(), // Generate unique ID for each transaction
            sender,
            receiver,
            amount,
            parent_id,
            child_ids: Vec::new(),
            signature,
            timestamp,
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

    pub fn add_transaction(
        &mut self,
        sender_name: String,
        receiver_name: String,
        amount: u64,
        signature: Vec<u8>,
        timestamp: u64,
    ) -> Result<(), String> {
        let parent_id = self.latest_transaction_id.clone(); // The parent is the latest transaction

        let transaction = Transaction::new(
            sender_name.clone(),
            receiver_name.clone(),
            amount,
            parent_id.clone(),
            signature,
            timestamp,
        );

        // Clone the transaction ID before moving `transaction`
        let transaction_id = transaction.id.clone();

        // If there was a previous transaction, update it to link to this new one as a child
        if let Some(parent_id) = parent_id {
            if let Some(parent_transaction) = self.transactions.get_mut(&parent_id) {
                parent_transaction.child_ids.push(transaction_id.clone());
            }
        }

        // Add the new transaction to the local DAG
        self.transactions.insert(transaction_id.clone(), transaction);

        // Update the latest transaction ID to the new transaction
        self.latest_transaction_id = Some(transaction_id);

        Ok(())
    }

    pub fn print_dag_in_order(&self) {
        print!("Heelloooo");
        // Find the first transaction (the one with no parent)
        let mut current_transaction = self
            .transactions
            .values()
            .find(|&tx| tx.parent_id.is_none());

        // If we found the root transaction, start traversing
        while let Some(transaction) = current_transaction {
            // Print the current transaction details
            println!(
                "Transaction ID: {}, Sender: {}, Receiver: {}, Amount: {}, Timestamp: {}, Signature: {:?}, Parent: {:?}, Children: {:?}",
                transaction.id,
                transaction.sender,
                transaction.receiver,
                transaction.amount,
                transaction.timestamp,
                transaction.signature,
                transaction.parent_id,
                transaction.child_ids
            );

            // Move to the next transaction, which should be the first child (if any)
            if !transaction.child_ids.is_empty() {
                current_transaction = self.transactions.get(&transaction.child_ids[0]);
            } else {
                current_transaction = None; // No more children, stop traversal
            }
        }
    }

    pub fn get_transaction(&self, transaction_id: &str) -> Option<&Transaction> {
        self.transactions.get(transaction_id)
    }
}
