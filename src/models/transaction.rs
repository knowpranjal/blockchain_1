use crate::DAGs::transaction_dag::{DAG, BlockTransaction}; // Import the DAG and Transaction
use crate::models::user::UserPool;
use crate::DAGs::user_DAG::LocalDAG; // Import the local DAG
use crate::models::pedersen::{create_range_proof, verify_range_proof};
use std::sync::{Arc, Mutex};
use std::io::Write;
use std::net::TcpStream;

pub fn process_transactions(
    transactions_data: Vec<(String, String, u64)>,
    user_pool: Arc<Mutex<UserPool>>,
    dag: Arc<Mutex<DAG>>,
    stream: &mut TcpStream,
) {
    let mut pool = user_pool.lock().unwrap();
    let mut dag_transactions = Vec::new(); // Collect transactions for the DAG

    for (sender_name, receiver_name, amount) in transactions_data {
        if !pool.user_exists(&sender_name) || !pool.user_exists(&receiver_name) {
            let _ = stream.write(format!("Error: User {} or {} does not exist\n", sender_name, receiver_name).as_bytes());
            continue;
        }

        // Process the transaction
        let sender_new_balance;
        let receiver_new_balance;

        {
            let sender = pool.get_user(&sender_name).unwrap();
            if sender.wallet.balance < amount {
                let _ = stream.write(format!("Error: Insufficient balance for user {}\n", sender_name).as_bytes());
                continue;
            }

            sender.wallet.balance -= amount;
            sender_new_balance = sender.wallet.balance;
            sender.local_dag.add_transaction(sender_name.clone(), receiver_name.clone(), amount).unwrap();
        }

        {
            let receiver = pool.get_user(&receiver_name).unwrap();
            receiver.wallet.balance += amount;
            receiver_new_balance = receiver.wallet.balance;
            receiver.local_dag.add_transaction(sender_name.clone(), receiver_name.clone(), amount).unwrap();
        }

        // Create a new transaction for the DAG
        let dag_transaction = BlockTransaction::new(
            sender_name.clone(),
            receiver_name.clone(),
            amount,
        );

        dag_transactions.push(dag_transaction);

        // Log the transaction
        println!(
            "{} sent {} tokens to {}. New balances -> {}: {}, {}: {}",
            sender_name, amount, receiver_name,
            sender_name, sender_new_balance,
            receiver_name, receiver_new_balance
        );
    }

    // Add transactions to the DAG
    if !dag_transactions.is_empty() {
        let mut dag = dag.lock().unwrap();
        match dag.add_transactions(dag_transactions) {
            Ok(_) => {
                let _ = stream.write(b"Transactions added to DAG successfully\n");
            }
            Err(e) => {
                let _ = stream.write(format!("Error adding transactions to DAG: {}\n", e).as_bytes());
            }
        }
    } else {
        let _ = stream.write(b"No valid transactions to process\n");
    }
}