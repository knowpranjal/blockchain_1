use crate::chains::blockchain::Blockchain;
use crate::models::user::UserPool;
use crate::models::pedersen::{create_range_proof, verify_range_proof};
use std::sync::{Arc, Mutex};
use std::io::Write;
use std::net::TcpStream;
use sha2::{Sha256, Digest};


fn generate_transaction_hash(sender: &str, receiver: &str, amount: u64) -> String {
    let mut hasher = Sha256::new();
    hasher.update(sender);
    hasher.update(receiver);
    hasher.update(amount.to_string());
    format!("{:x}", hasher.finalize())
}

// Handles the actual transaction logic
pub fn process_transaction(
    sender_name: &str,
    receiver_name: &str,
    amount: u64,
    user_pool: Arc<Mutex<UserPool>>,
    blockchain: Arc<Mutex<Blockchain>>,
    mainchain: Arc<Mutex<Blockchain>>,
    stream: &mut TcpStream,
) {
    let mut pool = user_pool.lock().unwrap();

    if !pool.user_exists(sender_name) || !pool.user_exists(receiver_name) {
        let _ = stream.write(b"Error: One or both users do not exist\n");
        return;
    }

    let sender_new_balance;
    let receiver_new_balance;

    // First, get the sender and perform necessary checks
    {
        let sender = pool.get_user(sender_name).unwrap();
        if sender.wallet.balance < amount {
            let _ = stream.write(b"Error: Insufficient balance\n");
            return;
        }

        
        sender.wallet.balance -= amount;
        sender_new_balance = sender.wallet.balance;
        // Deduct the amount from the sender and store the new balance
        
    }

    // Now, get the receiver and add the amount
    {
        let receiver = pool.get_user(receiver_name).unwrap();
        let (proof, commitment) = create_range_proof(amount);
        if verify_range_proof(proof, commitment) {
            receiver.wallet.balance += amount;
        }
        receiver_new_balance = receiver.wallet.balance;
    }

    // After the transaction, respond and update the blockchain
    let mut chain = blockchain.lock().unwrap();
    let transaction_info = format!("{} sent {} tokens to {}", sender_name, amount, receiver_name);
    chain.add_block(transaction_info.clone());

    if let Err(e) = stream.write(b"Transaction successful\n") {
        eprintln!("Failed to send response: {}", e);
        return;
    }

    let transaction_hash = generate_transaction_hash(sender_name, receiver_name, amount);

    // Add the transaction hash to the mainchain
    let mut mainchain = mainchain.lock().unwrap();
    mainchain.add_block(transaction_hash.clone());

    // Existing transaction print logic...
    println!("Transaction hash added to mainchain: {}", transaction_hash);

    // Use the stored balances instead of re-borrowing `pool`
    println!(
        "{} sent {} tokens to {}. New balances -> {}: {}, {}: {}",
        sender_name, amount, receiver_name,
        sender_name, sender_new_balance,
        receiver_name, receiver_new_balance
    );
}
