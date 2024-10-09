use crate::DAGs::transaction_dag::{DAG, BlockTransaction}; // Import the DAG and Transaction
use crate::models::user::UserPool;
use crate::models::pki::KeyPairWrapper;
use std::sync::{Arc, Mutex};
use std::io::Write;
use std::net::TcpStream;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid; // Add this import at the top of `transaction.rs`
use crate::models::persistence::{save_user_pool_state, save_dag_state};


pub fn process_transactions(
    transactions_data: Vec<(String, String, String, u64)>,
    user_pool: Arc<Mutex<UserPool>>,
    dag: Arc<Mutex<DAG>>,
    stream: &mut TcpStream,
) {
    let mut dag_transactions = Vec::new(); // Collect transactions for the DAG

    {
        // Start of `user_pool` lock scope
        let mut pool = user_pool.lock().unwrap();

        for (tx_type, sender_name, receiver_name, amount) in transactions_data {
            if tx_type != "TOKEN" {
                let _ = stream.write(
                    format!(
                        "Error: Transaction type {} is not specified as of now\n",
                        tx_type
                    )
                    .as_bytes(),
                );
                continue;
            }

            if !pool.user_exists(&sender_name) || !pool.user_exists(&receiver_name) {
                let _ = stream.write(
                    format!(
                        "Error: User {} or {} does not exist\n",
                        sender_name, receiver_name
                    )
                    .as_bytes(),
                );
                continue;
            }

            // Generate a unique transaction ID
            let transaction_id = Uuid::new_v4().to_string();

            // First, process the sender in a separate scope
            let (signature, timestamp) = {
                let sender = pool.get_user_mut(&sender_name).unwrap();

                if sender.wallet.balance < amount {
                    let _ = stream.write(
                        format!("Error: Insufficient balance for user {}\n", sender_name).as_bytes(),
                    );
                    continue;
                }

                // Generate timestamp
                let start = SystemTime::now();
                let timestamp = start.duration_since(UNIX_EPOCH).unwrap().as_secs();

                // Create a message to sign, including the transaction ID
                let message = format!(
                    "{}:{}:{}:{}:{}",
                    transaction_id, sender_name, receiver_name, amount, timestamp
                );

                // Sign the message using the sender's private key
                let signature = match sender.key_pair_wrapper.sign(message.as_bytes()) {
                    Ok(sig) => sig,
                    Err(e) => {
                        let _ = stream.write(
                            format!(
                                "Error: Failed to sign transaction for user {}: {}\n",
                                sender_name, e
                            )
                            .as_bytes(),
                        );
                        continue;
                    }
                };

                // Verify the signature using the sender's public key
                if let Err(e) = KeyPairWrapper::verify(
                    &sender.public_key,
                    message.as_bytes(),
                    signature.as_ref(),
                ) {
                    let _ = stream.write(
                        format!(
                            "Error: Signature verification failed for user {}: {}\n",
                            sender_name, e
                        )
                        .as_bytes(),
                    );
                    continue;
                }

                // Update sender's balance
                sender.wallet.balance -= amount;

                // Add transaction to sender's local DAG
                if let Err(e) = sender.local_dag.add_transaction(
                    transaction_id.clone(), // Use the generated transaction ID
                    sender_name.clone(),
                    receiver_name.clone(),
                    amount,
                    signature.as_ref().to_vec(),
                    timestamp,
                ) {
                    let _ = stream.write(
                        format!(
                            "Error: Failed to add transaction to sender's DAG: {}\n",
                            e
                        )
                        .as_bytes(),
                    );
                    continue;
                }

                (signature.as_ref().to_vec(), timestamp)
            };

            // Now, process the receiver in a separate scope
            {
                let receiver = pool.get_user_mut(&receiver_name).unwrap();

                // Update receiver's balance
                receiver.wallet.balance += amount;

                // Add transaction to receiver's local DAG
                if let Err(e) = receiver.local_dag.add_transaction(
                    transaction_id.clone(), // Use the same transaction ID
                    sender_name.clone(),
                    receiver_name.clone(),
                    amount,
                    signature.clone(),
                    timestamp,
                ) {
                    let _ = stream.write(
                        format!(
                            "Error: Failed to add transaction to receiver's DAG: {}\n",
                            e
                        )
                        .as_bytes(),
                    );
                    continue;
                }
            }

            // Create a new transaction for the DAG
            let dag_transaction = BlockTransaction::new(
                transaction_id.clone(), // Use the same transaction ID
                sender_name.clone(),
                receiver_name.clone(),
                amount,
                signature,
                timestamp,
            );

            dag_transactions.push(dag_transaction);

            // Log the transaction
            println!(
                "{} sent {} tokens to {}. New balances -> {}: {}, {}: {}",
                sender_name,
                amount,
                receiver_name,
                sender_name,
                pool.get_user(&sender_name).unwrap().wallet.balance,
                receiver_name,
                pool.get_user(&receiver_name).unwrap().wallet.balance
            );
        } // End of `for` loop
    } // End of `user_pool` lock scope

    // Now, lock the `dag` mutex without holding the `user_pool` lock
    if !dag_transactions.is_empty() {
        let mut dag = dag.lock().unwrap();
        match dag.add_transactions(dag_transactions) {
            Ok(_) => {
                let _ = stream.write(b"Transactions added to DAG successfully\n");
                stream.flush().unwrap();
                
                // Save the updated DAG state
                if let Err(e) = save_dag_state(&dag) {
                    eprintln!("Failed to save DAG state: {}", e);
                }

                // Save the updated UserPool state
                let pool = user_pool.lock().unwrap();
                if let Err(e) = save_user_pool_state(&pool) {
                    eprintln!("Failed to save UserPool state: {}", e);
                }
            }
            Err(e) => {
                let _ = stream.write(format!("Error adding transactions to DAG: {}\n", e).as_bytes());
                stream.flush().unwrap();
            }
        }
    } else {
        let _ = stream.write(b"No valid transactions to process\n");
        stream.flush().unwrap();
    }
}

