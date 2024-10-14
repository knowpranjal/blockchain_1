use std::collections::HashMap;
use uuid::Uuid;
use std::collections::HashSet;
use crate::models::pki::KeyPairWrapper;
use crate::models::user::UserPool;
use serde::{Serialize, Deserialize};

use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Transaction {
    pub id: String,
    pub sender: String,
    pub receiver: String,
    pub amount: u64,
    pub parent_id: Option<String>, // Parent transaction ID
    pub child_ids: Vec<String>,    // Children transaction IDs
    pub signature: Vec<u8>,        // Signature
    pub timestamp: u64,            // Timestamp
}

impl Transaction {
    pub fn new(
        id: String, // Use provided transaction ID
        sender: String,
        receiver: String,
        amount: u64,
        parent_id: Option<String>,
        signature: Vec<u8>,
        timestamp: u64,
    ) -> Self {
        Transaction {
            id, // Use the provided ID
            sender,
            receiver,
            amount,
            parent_id,
            child_ids: Vec::new(),
            signature,
            timestamp,
        }
    }

    pub fn compute_hash(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(self.id.as_bytes());
        hasher.update(self.sender.as_bytes());
        hasher.update(self.receiver.as_bytes());
        hasher.update(&self.amount.to_le_bytes());
        hasher.update(&self.timestamp.to_le_bytes());
        hasher.update(&self.signature);
        hasher.finalize().to_vec()
    }
}




#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
        transaction_id: String, // Accept the transaction ID
        sender_name: String,
        receiver_name: String,
        amount: u64,
        signature: Vec<u8>,
        timestamp: u64,
    ) -> Result<(), String> {
        let parent_id = self.latest_transaction_id.clone(); // The parent is the latest transaction

        let transaction = Transaction::new(
            transaction_id.clone(), // Use the provided transaction ID
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

    pub fn get_transaction_by_id(&self, transaction_id: &str) -> Option<&Transaction> {
        self.transactions.get(transaction_id)
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

    pub fn get_dag_as_string(&self) -> String {
        let mut output = String::new();
        // Find all root transactions (those with no parent)
        let root_transactions: Vec<&Transaction> = self
            .transactions
            .values()
            .filter(|&tx| tx.parent_id.is_none())
            .collect();

        // Traverse each root transaction
        for root_tx in root_transactions {
            self.traverse_and_build_string(root_tx, &mut output, 0);
        }

        output
    }

    fn traverse_and_build_string(
        &self,
        transaction: &Transaction,
        output: &mut String,
        depth: usize,
    ) {
        let indent = "  ".repeat(depth);
        output.push_str(&format!(
            "{}Transaction ID: {}, Sender: {}, Receiver: {}, Amount: {}, Timestamp: {}, Signature: {:?}, Parent: {:?}, Children: {:?}\n",
            indent,
            transaction.id,
            transaction.sender,
            transaction.receiver,
            transaction.amount,
            transaction.timestamp,
            transaction.signature,
            transaction.parent_id,
            transaction.child_ids
        ));

        for child_id in &transaction.child_ids {
            if let Some(child_tx) = self.transactions.get(child_id) {
                self.traverse_and_build_string(child_tx, output, depth + 1);
            }
        }
    }

    pub fn validate_transactions(
        &self,
        user_name: &str,
        user_public_key: &[u8],
        user_pool: &UserPool,
    ) -> Result<(), String> {
        let mut visited = HashSet::new();

        // Find root transactions (transactions with no parent)
        let root_transactions: Vec<&Transaction> = self.transactions.values()
            .filter(|tx| tx.parent_id.is_none())
            .collect();

        // Starting balance for validation
        let initial_balance = user_pool.get_user_initial_balance(user_name).unwrap_or(0);
        let mut balance = initial_balance;

        // Traverse each root transaction
        for root_tx in root_transactions {
            self.validate_transaction_recursive(
                root_tx,
                user_name,
                user_pool,
                &mut visited,
                &mut balance,
            )?;
        }

        Ok(())
    }

    fn validate_transaction_recursive(
        &self,
        transaction: &Transaction,
        user_name: &str,
        user_pool: &UserPool,
        visited: &mut HashSet<String>,
        balance: &mut u64,
    ) -> Result<(), String> {
        if visited.contains(&transaction.id) {
            // Transaction already validated
            return Ok(());
        }

        // Mark transaction as visited
        visited.insert(transaction.id.clone());

        // Verify the transaction's signature
        let message = format!(
            "{}:{}:{}:{}:{}",
            transaction.id,
            transaction.sender,
            transaction.receiver,
            transaction.amount,
            transaction.timestamp,
        );

        // Get sender's public key
        let sender_public_key = user_pool
            .get_user_public_key(&transaction.sender)
            .ok_or(format!("Public key not found for user {}", transaction.sender))?;

        if let Err(e) = KeyPairWrapper::verify(
            &sender_public_key,
            message.as_bytes(),
            &transaction.signature,
        ) {
            return Err(format!(
                "Invalid signature for transaction {}: {:?}",
                transaction.id, e
            ));
        }

        // If the transaction has a parent, validate the parent first
        if let Some(parent_id) = &transaction.parent_id {
            if let Some(parent_tx) = self.transactions.get(parent_id) {
                self.validate_transaction_recursive(
                    parent_tx,
                    user_name,
                    user_pool,
                    visited,
                    balance,
                )?;
            } else {
                return Err(format!(
                    "Parent transaction {} not found for transaction {}",
                    parent_id, transaction.id
                ));
            }
        }

        // Update balance
        // If the user is the sender, subtract amount
        if transaction.sender == user_name {
            if *balance < transaction.amount {
                return Err(format!(
                    "Insufficient balance for transaction {}",
                    transaction.id
                ));
            }
            *balance -= transaction.amount;
        }

        // If the user is the receiver, add amount
        if transaction.receiver == user_name {
            *balance += transaction.amount;
        }

        Ok(())
    }
}
