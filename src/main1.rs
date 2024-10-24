// main.rs

mod core;
mod chains;
mod models;
mod DAGs;

use crate::chains::blockchain::Blockchain;
use crate::DAGs::transaction_dag::DAG;
use crate::models::user::{User, UserPool};
use crate::models::transaction::{process_transactions, finalize_transaction, PendingTransaction};
use crate::models::pki::KeyPairWrapper;
use crate::models::persistence::{save_user_pool_state, load_user_pool_state, save_dag_state, load_dag_state};

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use std::str;

const NODE_B_ADDRESS: &str = "192.168.0.121:8081"; // Replace with Node B's IP and port

async fn handle_client(
    mut stream: TcpStream,
    user_pool: Arc<RwLock<UserPool>>,
    dag: Arc<Mutex<DAG>>,
) {
    let mut buffer = [0; 4096];

    loop {
        match stream.read(&mut buffer).await {
            Ok(0) => return, // Connection closed by client
            Ok(bytes_read) => {
                let request = match str::from_utf8(&buffer[..bytes_read]) {
                    Ok(v) => v.trim(),
                    Err(_) => {
                        let _ = stream.write_all(b"Error: Invalid UTF-8\n").await;
                        continue;
                    }
                };
                println!("Received request: {}", request);

                match TcpStream::connect(NODE_B_ADDRESS).await {
                    Ok(mut node_b_stream) => {
                        if let Err(e) = node_b_stream.write_all(request.as_bytes()).await {
                            eprintln!("Failed to send request to Node B: {}", e);
                        }
                        // Optionally read response from Node B
                        let mut response_buffer = [0; 4096];
                        match node_b_stream.read(&mut response_buffer).await {
                            Ok(bytes_read) => {
                                if bytes_read > 0 {
                                    let response = match str::from_utf8(&response_buffer[..bytes_read]) {
                                        Ok(v) => v.trim(),
                                        Err(_) => "Error: Invalid UTF-8 in Node B response",
                                    };
                                    println!("Received response from Node B: {}", response);
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to read response from Node B: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to connect to Node B: {}", e);
                    }
                }

                if request.starts_with("VIEW_PENDING_TRANSACTIONS") {
                    let user_name = request.replace("VIEW_PENDING_TRANSACTIONS ", "").trim().to_string();
                    let pool = user_pool.read().await;
                    if !pool.user_exists(&user_name) {
                        let _ = stream.write_all(b"Error: User does not exist\n").await;
                        continue;
                    }
                    let pending_txs: Vec<_> = pool.pending_transactions.values()
                        .filter(|tx| tx.receiver == user_name)
                        .cloned()
                        .collect();
                    drop(pool); // Release lock

                    if pending_txs.is_empty() {
                        let _ = stream.write_all(b"No pending transactions\n").await;
                    } else {
                        for tx in pending_txs {
                            let _ = stream.write_all(
                                format!(
                                    "Pending Transaction ID: {}, From: {}, Amount: {}\n",
                                    tx.id, tx.sender, tx.amount
                                )
                                .as_bytes(),
                            ).await;
                        }
                    }
                    let _ = stream.flush().await;
                } else if request.starts_with("CONFIRM_TRANSACTION") {
                    let parts: Vec<&str> = request.split_whitespace().collect();
                    if parts.len() != 3 {
                        let _ = stream.write_all(b"Usage: CONFIRM_TRANSACTION <UserName> <TransactionID>\n").await;
                        continue;
                    }
                    let user_name = parts[1];
                    let transaction_id = parts[2];

                    // Lock user_pool briefly to get and remove pending_tx
                    let pending_tx = {
                        let mut pool = user_pool.write().await;
                        let pending_tx = match pool.pending_transactions.remove(transaction_id) {
                            Some(tx) => tx.clone(),
                            None => {
                                let _ = stream.write_all(b"Error: Transaction not found\n").await;
                                continue;
                            }
                        };
                        if pending_tx.receiver != user_name {
                            let _ = stream.write_all(b"Error: Transaction is not pending for this user\n").await;
                            // Re-insert pending_tx into pending_transactions
                            pool.pending_transactions.insert(transaction_id.to_string(), pending_tx);
                            continue;
                        }
                        // Save the updated UserPool state
                        if let Err(e) = save_user_pool_state(&pool) {
                            eprintln!("Failed to save UserPool state: {}", e);
                        }
                        pending_tx
                    }; // Release the lock on user_pool

                    // Proceed to finalize the transaction
                    let result = finalize_transaction(
                        pending_tx.clone(),
                        Arc::clone(&user_pool),
                        Arc::clone(&dag),
                    ).await;

                    match result {
                        Ok(_) => {
                            let _ = stream.write_all(b"Transaction confirmed and processed\n").await;
                        }
                        Err(e) => {
                            let _ = stream.write_all(format!("Error processing transaction: {}\n", e).as_bytes()).await;
                        }
                    }

                    let _ = stream.flush().await;
                } else if request.starts_with("REJECT_TRANSACTION") {
                    let parts: Vec<&str> = request.split_whitespace().collect();
                    if parts.len() != 3 {
                        let _ = stream.write_all(b"Usage: REJECT_TRANSACTION <UserName> <TransactionID>\n").await;
                        continue;
                    }
                    let user_name = parts[1];
                    let transaction_id = parts[2];
                    let mut pool = user_pool.write().await;
                    let pending_tx = match pool.pending_transactions.get(transaction_id) {
                        Some(tx) => tx.clone(),
                        None => {
                            let _ = stream.write_all(b"Error: Transaction not found\n").await;
                            continue;
                        }
                    };
                    if pending_tx.receiver != user_name {
                        let _ = stream.write_all(b"Error: Transaction is not pending for this user\n").await;
                        continue;
                    }

                    // Remove from pending transactions
                    pool.pending_transactions.remove(transaction_id);
                    let _ = stream.write_all(b"Transaction rejected\n").await;

                    // Save the updated UserPool state
                    if let Err(e) = save_user_pool_state(&pool) {
                        eprintln!("Failed to save UserPool state: {}", e);
                    }
                    let _ = stream.flush().await;
                } else if request.starts_with("VALIDATE_LOCAL_DAG") {
                    let user_name = request.replace("VALIDATE_LOCAL_DAG ", "").trim().to_string();
                    let pool = user_pool.read().await;
                    if let Some(user) = pool.get_user(&user_name) {
                        match user.validate_local_dag(&pool) {
                            Ok(_) => {
                                let _ = stream.write_all(b"Local DAG is valid\n").await;
                            }
                            Err(e) => {
                                let _ = stream.write_all(format!("Local DAG validation failed: {}\n", e).as_bytes()).await;
                            }
                        }
                        let _ = stream.flush().await;
                    } else {
                        let _ = stream.write_all(b"Error: User does not exist\n").await;
                        let _ = stream.flush().await;
                    }
                } else if request.starts_with("QUERY_TRANSACTION") {
                    let transaction_id = request.replace("QUERY_TRANSACTION ", "").trim().to_string();
                    let dag = dag.lock().await;
                    if let Some(transaction) = dag.get_transaction(&transaction_id) {
                        let response = format!(
                            "Transaction found: ID: {}, Sender: {}, Receiver: {}, Amount: {}, Timestamp: {}, Signature: {:?}\n",
                            transaction.id,
                            transaction.sender,
                            transaction.receiver,
                            transaction.amount,
                            transaction.timestamp,
                            transaction.signature
                        );
                        let _ = stream.write_all(response.as_bytes()).await;
                    } else {
                        let _ = stream.write_all(format!("Transaction with ID {} not found\n", transaction_id).as_bytes()).await;
                    }
                    let _ = stream.flush().await;
                } else if request.starts_with("FETCH_USER_DAGS") {
                    let user_names_str = request.replace("FETCH_USER_DAGS ", "");
                    let user_names: Vec<&str> = user_names_str.split_whitespace().collect();
                    let pool = user_pool.read().await;

                    let mut response = String::new();

                    for user_name in user_names {
                        if let Some(user) = pool.get_user(user_name) {
                            response.push_str(&format!("User {}'s DAG:\n", user_name));
                            response.push_str(&user.local_dag.get_dag_as_string());
                            response.push_str("\n");
                        } else {
                            response.push_str(&format!("Error: User {} does not exist\n", user_name));
                        }
                    }

                    drop(pool); // Release lock

                    if let Err(e) = stream.write_all(response.as_bytes()).await {
                        eprintln!("Failed to send response: {}", e);
                    }
                    let _ = stream.flush().await;
                } else if request.starts_with("CHECK_BALANCE") {
                    let user_name = request.replace("CHECK_BALANCE ", "").trim().to_string();
                    let pool = user_pool.read().await;

                    if let Some(user) = pool.get_user(&user_name) {
                        let balance = user.get_balance();
                        let _ = stream.write_all(
                            format!("User {} has a balance of {}\n", user_name, balance).as_bytes(),
                        ).await;
                        let _ = stream.flush().await;
                    } else {
                        let _ = stream.write_all(b"Error: User does not exist\n").await;
                        let _ = stream.flush().await;
                    }
                } else if request.starts_with("TRANSACTION") {
                    let transaction_data = request.replace("TRANSACTION ", "");
                    let transactions = parse_transaction_data(&transaction_data);

                    if transactions.is_empty() {
                        let _ = stream.write_all(b"Error: Invalid transaction data\n").await;
                        continue;
                    }

                    process_transactions(
                        transactions,
                        Arc::clone(&user_pool),
                        Arc::clone(&dag),
                        &mut stream,
                    ).await;

                } else if request.starts_with("ADD_USER") {
                    let user_data = request.replace("ADD_USER ", "");
                    let (name, balance_str) = parse_add_user_data(&user_data);

                    let balance: u64 = match balance_str.parse() {
                        Ok(val) => val,
                        Err(_) => {
                            let _ = stream.write_all(b"Error: Invalid balance\n").await;
                            continue;
                        }
                    };

                    let mut pool = user_pool.write().await;
                    if pool.user_exists(&name) {
                        let _ = stream.write_all(b"Error: User already exists\n").await;
                    } else {
                        let user = User::new(name.clone(), balance);
                        let public_key = user.public_key.clone();
                        pool.add_user(user);
                        let _ = stream.write_all(
                            format!(
                                "User {} added with balance {} and public key {:?}\n",
                                name,
                                balance,
                                public_key
                            ).as_bytes(),
                        ).await;
                        // Save the updated UserPool state
                        if let Err(e) = save_user_pool_state(&pool) {
                            eprintln!("Failed to save UserPool state: {}", e);
                        }
                    }
                } else if request.starts_with("PRINT_DAG") {
                    let dag = dag.lock().await;
                    let mut response = String::new();
                    response.push_str("Blockchain DAG:\n");
                    for block in dag.blocks.values() {
                        response.push_str(&format!("Block ID: {}\n", block.id));
                        response.push_str(&format!("  Parents: {:?}\n", block.parent_ids));
                        response.push_str(&format!("  Children: {:?}\n", block.child_ids));
                        response.push_str(&format!(
                            "  Transactions: {:?}\n",
                            block.transactions.iter().map(|tx| (&tx.id, &tx.sender, &tx.receiver, tx.amount)).collect::<Vec<_>>()
                        ));
                        response.push_str("\n");
                    }
                    if let Err(e) = stream.write_all(response.as_bytes()).await {
                        eprintln!("Failed to send response: {}", e);
                    }
                } else if request.starts_with("PRINT_USER_DAG") {
                    println!("Command received");
                    let user_name = request.replace("PRINT_USER_DAG ", "").trim().to_string();
                    let pool = user_pool.read().await;

                    if let Some(user) = pool.get_user(&user_name) {
                        // Call the print_dag method from LocalDAG
                        user.local_dag.print_dag_in_order();
                        let _ = stream.write_all(format!("User {}'s DAG printed to the log\n", user_name).as_bytes()).await;
                    } else {
                        let _ = stream.write_all(b"Error: User does not exist\n").await;
                    }
                } else if request.starts_with("VERIFY_TRANSACTION") {
                    let transaction_id = request.replace("VERIFY_TRANSACTION ", "").trim().to_string();
                    let dag = dag.lock().await;
                    let global_tx = match dag.get_transaction(&transaction_id) {
                        Some(tx) => tx,
                        None => {
                            let _ = stream.write_all(format!("Transaction with ID {} not found in global DAG\n", transaction_id).as_bytes()).await;
                            let _ = stream.flush().await;
                            continue;
                        }
                    };

                    // Get the sender and receiver names from the transaction
                    let sender_name = global_tx.sender.clone();
                    let receiver_name = global_tx.receiver.clone();

                    let pool = user_pool.read().await;

                    // Retrieve the transaction from the sender's local DAG
                    let sender_tx = match pool.get_user(&sender_name)
                        .and_then(|user| user.local_dag.get_transaction_by_id(&transaction_id)) {
                        Some(tx) => tx,
                        None => {
                            let _ = stream.write_all(format!("Transaction not found in sender {}'s DAG\n", sender_name).as_bytes()).await;
                            let _ = stream.flush().await;
                            continue;
                        }
                    };

                    // Retrieve the transaction from the receiver's local DAG
                    let receiver_tx = match pool.get_user(&receiver_name)
                        .and_then(|user| user.local_dag.get_transaction_by_id(&transaction_id)) {
                        Some(tx) => tx,
                        None => {
                            let _ = stream.write_all(format!("Transaction not found in receiver {}'s DAG\n", receiver_name).as_bytes()).await;
                            let _ = stream.flush().await;
                            continue;
                        }
                    };

                    // Compute hashes
                    let sender_hash = sender_tx.compute_hash();
                    let receiver_hash = receiver_tx.compute_hash();
                    let hashes_match = sender_hash == receiver_hash;

                    // Verify the signature
                    let message = format!(
                        "{}:{}:{}:{}:{}",
                        global_tx.id, global_tx.sender, global_tx.receiver, global_tx.amount, global_tx.timestamp
                    );
                    let sender_public_key = match pool.get_user_public_key(&global_tx.sender) {
                        Some(pk) => pk,
                        None => {
                            let _ = stream.write_all(format!("Sender public key not found for {}\n", global_tx.sender).as_bytes()).await;
                            let _ = stream.flush().await;
                            continue;
                        }
                    };

                    let signature_valid = KeyPairWrapper::verify(
                        &sender_public_key,
                        message.as_bytes(),
                        &global_tx.signature,
                    ).is_ok();

                    if hashes_match && signature_valid {
                        let _ = stream.write_all(b"Transaction integrity verified successfully\n").await;
                    } else {
                        let _ = stream.write_all(b"Transaction integrity verification failed\n").await;
                    }
                    let _ = stream.flush().await;
                } else {
                    if let Err(e) = stream.write_all(b"Unknown command\n").await {
                        eprintln!("Failed to send response: {}", e);
                        return;
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read from stream: {}", e);
                return;
            }
        }
    }
}

// Helper functions

fn parse_transaction_data(data: &str) -> Vec<(String, String, String, u64)> {
    let parts: Vec<&str> = data.split_whitespace().collect();
    let mut transactions = Vec::new();

    if parts.len() % 4 != 0 {
        return transactions; // Return empty if data is invalid
    }

    for chunk in parts.chunks(4) {
        let tx_type = chunk[0].to_string();
        let sender = chunk[1].to_string();
        let receiver = chunk[2].to_string();
        let amount = match chunk[3].parse() {
            Ok(val) => val,
            Err(_) => continue, // Skip invalid transactions
        };
        transactions.push((tx_type, sender, receiver, amount));
    }

    transactions
}

fn parse_add_user_data(data: &str) -> (String, String) {
    let parts: Vec<&str> = data.split_whitespace().collect();
    if parts.len() == 2 {
        (parts[0].to_string(), parts[1].to_string())
    } else {
        ("".to_string(), "0".to_string()) // Return empty or default values in case of incorrect input
    }
}

#[tokio::main]
async fn main() {
    // Load UserPool state
    let user_pool = if let Some(pool) = load_user_pool_state() {
        Arc::new(RwLock::new(pool))
    } else {
        Arc::new(RwLock::new(UserPool::new()))
    };

    // Load DAG state
    let dag = if let Some(loaded_dag) = load_dag_state(Arc::clone(&user_pool)) {
        Arc::new(Mutex::new(loaded_dag))
    } else {
        Arc::new(Mutex::new(DAG::new(Arc::clone(&user_pool))))
    };

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Server listening on port 8080");

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                let user_pool = Arc::clone(&user_pool);
                let dag = Arc::clone(&dag);  // Pass the DAG

                tokio::spawn(async move {
                    handle_client(stream, user_pool, dag).await;  // Pass DAG to handle_client
                });
            }
            Err(e) => eprintln!("Failed to accept connection: {}", e),
        }
    }
}
