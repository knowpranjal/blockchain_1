use crate::blockchain::Blockchain;
use crate::user::UserPool;
use std::sync::{Arc, Mutex};
use std::io::Write;
use std::net::TcpStream;

// Handles the actual transaction logic
pub fn process_transaction(
    sender_name: &str,
    receiver_name: &str,
    amount: u64,
    user_pool: Arc<Mutex<UserPool>>,
    blockchain: Arc<Mutex<Blockchain>>,
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

        // Deduct the amount from the sender and store the new balance
        sender.wallet.balance -= amount;
        sender_new_balance = sender.wallet.balance;
    }

    // Now, get the receiver and add the amount
    {
        let receiver = pool.get_user(receiver_name).unwrap();
        receiver.wallet.balance += amount;
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

    // Use the stored balances instead of re-borrowing `pool`
    println!(
        "{} sent {} tokens to {}. New balances -> {}: {}, {}: {}",
        sender_name, amount, receiver_name,
        sender_name, sender_new_balance,
        receiver_name, receiver_new_balance
    );
}
