use crate::DAGs::transaction_dag::{DAG, Transaction}; // Import the DAG and Transaction
use crate::models::user::UserPool;
use crate::DAGs::user_DAG::LocalDAG; // Import the local DAG
use crate::models::pedersen::{create_range_proof, verify_range_proof};
use std::sync::{Arc, Mutex};
use std::io::Write;
use std::net::TcpStream;

pub fn process_transaction(
    sender_name: &str,
    receiver_name: &str,
    amount: u64,
    user_pool: Arc<Mutex<UserPool>>,
    dag: Arc<Mutex<DAG>>,
    stream: &mut TcpStream,
) {
    let mut pool = user_pool.lock().unwrap();

    if !pool.user_exists(sender_name) || !pool.user_exists(receiver_name) {
        let _ = stream.write(b"Error: One or both users do not exist\n");
        return;
    }

    // First, get the sender and perform necessary checks
    let sender_new_balance;
    {
        let sender = pool.get_user(sender_name).unwrap();
        if sender.wallet.balance < amount {
            let _ = stream.write(b"Error: Insufficient balance\n");
            return;
        }

        sender.wallet.balance -= amount;
        sender_new_balance = sender.wallet.balance;

        sender.local_dag.add_transaction(sender_name.to_string(), receiver_name.to_string(), amount).unwrap();
    }

    // Now, get the receiver and add the amount
    let receiver_new_balance;
    {
        let receiver = pool.get_user(receiver_name).unwrap();
        receiver.wallet.balance += amount;
        receiver_new_balance = receiver.wallet.balance;

        receiver.local_dag.add_transaction(sender_name.to_string(), receiver_name.to_string(), amount).unwrap();
    }

    // Update the local DAGs of both the sender and the receiver
    // {
    //     let sender = pool.get_user(sender_name).unwrap();
    //     let receiver = pool.get_user(receiver_name).unwrap();

        
        
    // }


    // Create a new transaction and add it to the DAG
    let mut dag = dag.lock().unwrap();
    let prev_transactions = vec![]; // In this case, no previous transactions. Update as needed.
    
    let transaction = Transaction::new(
        sender_name.to_string(),
        receiver_name.to_string(),
        amount,
        prev_transactions,
    );

    match dag.add_transaction(transaction.clone()) {
        Ok(_) => {
            let _ = stream.write(b"Transaction successful\n");
        }
        Err(e) => {
            let _ = stream.write(format!("Error: {}\n", e).as_bytes());
        }
    }

    // Existing transaction print logic
    println!(
        "{} sent {} tokens to {}. New balances -> {}: {}, {}: {}",
        sender_name, amount, receiver_name,
        sender_name, sender_new_balance,
        receiver_name, receiver_new_balance
    );
}
