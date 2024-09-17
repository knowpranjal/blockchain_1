use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: String,                  // Unique identifier for the transaction
    pub sender: String,              // Sender's public key
    pub receiver: String,            // Receiver's public key
    pub amount: u64,                 // Amount being transferred
    pub timestamp: u64,              // Timestamp of the transaction
    pub prev_transactions: Vec<String>, // References to previous transactions (transaction IDs)
}

impl Transaction {
    // Create a new transaction
    pub fn new(sender: String, receiver: String, amount: u64, prev_transactions: Vec<String>) -> Transaction {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let id = format!("{}{}{}{}", sender, receiver, amount, timestamp); // Simple ID for example purposes
        Transaction {
            id,
            sender,
            receiver,
            amount,
            timestamp,
            prev_transactions,
        }
    }
}


#[derive(Debug)]
pub struct DAG {
    pub transactions: HashMap<String, Transaction>,  // Map of transaction IDs to transactions
}

impl DAG {
    // Create a new empty DAG
    pub fn new() -> DAG {
        DAG {
            transactions: HashMap::new(),
        }
    }

    // Add a new transaction to the DAG
    pub fn add_transaction(&mut self, transaction: Transaction) -> Result<(), String> {
        // Check if the transaction creates a cycle
        if self.creates_cycle(&transaction) {
            return Err("Error: Adding this transaction would create a cycle".to_string());
        }

        // If valid, add the transaction to the DAG
        self.transactions.insert(transaction.id.clone(), transaction);
        Ok(())
    }

    // Check if a transaction creates a cycle in the DAG
    fn creates_cycle(&self, new_transaction: &Transaction) -> bool {
        let mut visited = vec![new_transaction.id.clone()];
        for prev_id in &new_transaction.prev_transactions {
            if self.has_cycle(prev_id, &mut visited) {
                return true;
            }
        }
        false
    }

    // Recursive helper function to check if there's a cycle
    fn has_cycle(&self, transaction_id: &String, visited: &mut Vec<String>) -> bool {
        // If we've already visited this transaction, there's a cycle
        if visited.contains(transaction_id) {
            return true;
        }

        // Mark the transaction as visited
        visited.push(transaction_id.clone());

        // Get the transaction and check its previous transactions
        if let Some(transaction) = self.transactions.get(transaction_id) {
            for prev_id in &transaction.prev_transactions {
                if self.has_cycle(prev_id, visited) {
                    return true;
                }
            }
        }

        // If no cycle found, remove the transaction from the visited list and return false
        visited.retain(|id| id != transaction_id);
        false
    }

    // Get a transaction by its ID
    pub fn get_transaction(&self, id: &String) -> Option<&Transaction> {
        self.transactions.get(id)
    }
}
