mod core;
mod chains;
mod models;
mod DAGs;

use crate::chains::blockchain::Blockchain;
use crate::DAGs::transaction_dag::DAG;
use crate::models::user::{User, UserPool};
use crate::models::transaction::process_transactions;
use crate::models::pki::KeyPairWrapper;

use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use std::sync::{Arc, Mutex};
use std::str;


fn handle_client(
    mut stream: TcpStream,
    user_pool: Arc<Mutex<UserPool>>,
    dag: Arc<Mutex<DAG>>, // Add DAG parameter
) {
    let mut buffer = [0; 4096];

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => return, // Connection closed by client
            Ok(bytes_read) => {
                let request = match str::from_utf8(&buffer[..bytes_read]) {
                    Ok(v) => v.trim(),
                    Err(_) => {
                        let _ = stream.write(b"Error: Invalid UTF-8\n");
                        continue;
                    }
                };
                println!("Received request: {}", request);

                if request.starts_with("QUERY_TRANSACTION") {
                    let transaction_id = request.replace("QUERY_TRANSACTION ", "").trim().to_string();
                    let dag = dag.lock().unwrap();
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
                        let _ = stream.write(response.as_bytes());
                    } else {
                        let _ = stream.write(format!("Transaction with ID {} not found\n", transaction_id).as_bytes());
                    }
                    let _ = stream.flush();
                }else if request.starts_with("FETCH_USER_DAGS") {
                    let user_names_str = request.replace("FETCH_USER_DAGS ", "");
                    let user_names: Vec<&str> = user_names_str.split_whitespace().collect();
                    let pool = user_pool.lock().unwrap();
    
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
    
                    if let Err(e) = stream.write(response.as_bytes()) {
                        eprintln!("Failed to send response: {}", e);
                    }
                    let _ = stream.flush();
                } else if request.starts_with("CHECK_BALANCE") {
                    let user_name = request.replace("CHECK_BALANCE ", "").trim().to_string();
                    let pool = user_pool.lock().unwrap();
    
                    if let Some(user) = pool.get_user(&user_name) {
                        let balance = user.get_balance();
                        let _ = stream.write(
                            format!("User {} has a balance of {}\n", user_name, balance).as_bytes(),
                        );
                        let _ = stream.flush();
                    } else {
                        let _ = stream.write(b"Error: User does not exist\n");
                        let _ = stream.flush();
                    }
                } else if request.starts_with("TRANSACTION") {
                    let transaction_data = request.replace("TRANSACTION ", "");
                    let transactions = parse_transaction_data(&transaction_data);

                    if transactions.is_empty() {
                        let _ = stream.write(b"Error: Invalid transaction data\n");
                        continue;
                    }

                    process_transactions(
                        transactions,
                        Arc::clone(&user_pool),
                        Arc::clone(&dag),
                        &mut stream,
                    );

                    print!("halla bol");
                } else if request.starts_with("ADD_USER") {
                    let user_data = request.replace("ADD_USER ", "");
                    let (name, balance_str) = parse_add_user_data(&user_data);
                
                    let balance: u64 = match balance_str.parse() {
                        Ok(val) => val,
                        Err(_) => {
                            let _ = stream.write(b"Error: Invalid balance\n");
                            continue;
                        }
                    };
                
                    let mut pool = user_pool.lock().unwrap();
                    if pool.user_exists(&name) {
                        let _ = stream.write(b"Error: User already exists\n");
                    } else {
                        let user = User::new(name.clone(), balance);
                        let public_key = user.public_key.clone();
                        pool.add_user(user);
                        let _ = stream.write(
                            format!(
                                "User {} added with balance {} and public key {:?}\n", 
                                name, 
                                balance, 
                                public_key
                            ).as_bytes(),
                        );
                    }
                } else if request.starts_with("PRINT_DAG") {

                    let dag = dag.lock().unwrap();
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
                    if let Err(e) = stream.write(response.as_bytes()) {
                        eprintln!("Failed to send response: {}", e);
                    }

                } else if request.starts_with("PRINT_USER_DAG") {
                    println!("Command received");
                    let user_name = request.replace("PRINT_USER_DAG ", "").trim().to_string();
                    let pool = user_pool.lock().unwrap();
                    
                    if let Some(user) = pool.get_user(&user_name) {
                        // Call the print_dag method from LocalDAG
                        user.local_dag.print_dag_in_order();
                        let _ = stream.write(format!("User {}'s DAG printed to the log\n", user_name).as_bytes());
                    } else {
                        let _ = stream.write(b"Error: User does not exist\n");
                    }
                } else if request.starts_with("VERIFY_TRANSACTION") {
                    let transaction_id = request.replace("VERIFY_TRANSACTION ", "").trim().to_string();
                    let dag = dag.lock().unwrap();
                    let global_tx = match dag.get_transaction(&transaction_id) {
                        Some(tx) => tx,
                        None => {
                            let _ = stream.write(format!("Transaction with ID {} not found in global DAG\n", transaction_id).as_bytes());
                            let _ = stream.flush();
                            continue;
                        }
                    };
                
                    // Get the sender and receiver names from the transaction
                    let sender_name = global_tx.sender.clone();
                    let receiver_name = global_tx.receiver.clone();
                
                    let pool = user_pool.lock().unwrap();
                
                    // Retrieve the transaction from the sender's local DAG
                    let sender_tx = match pool.get_user(&sender_name)
                        .and_then(|user| user.local_dag.get_transaction_by_id(&transaction_id)) {
                        Some(tx) => tx,
                        None => {
                            let _ = stream.write(format!("Transaction not found in sender {}'s DAG\n", sender_name).as_bytes());
                            let _ = stream.flush();
                            continue;
                        }
                    };
                
                    // Retrieve the transaction from the receiver's local DAG
                    let receiver_tx = match pool.get_user(&receiver_name)
                        .and_then(|user| user.local_dag.get_transaction_by_id(&transaction_id)) {
                        Some(tx) => tx,
                        None => {
                            let _ = stream.write(format!("Transaction not found in receiver {}'s DAG\n", receiver_name).as_bytes());
                            let _ = stream.flush();
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
                            let _ = stream.write(format!("Sender public key not found for {}\n", global_tx.sender).as_bytes());
                            let _ = stream.flush();
                            continue;
                        }
                    };
                
                    let signature_valid = KeyPairWrapper::verify(
                        &sender_public_key,
                        message.as_bytes(),
                        &global_tx.signature,
                    ).is_ok();
                
                    if hashes_match && signature_valid {
                        let _ = stream.write(b"Transaction integrity verified successfully\n");
                    } else {
                        let _ = stream.write(b"Transaction integrity verification failed\n");
                    }
                    let _ = stream.flush();
                }
                
                else {
                    if let Err(e) = stream.write(b"Unknown command\n") {
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

// Helper function to parse ADD_USER data
fn parse_add_user_data(data: &str) -> (String, String) {
    let parts: Vec<&str> = data.split_whitespace().collect();
    if parts.len() == 2 {
        (parts[0].to_string(), parts[1].to_string())
    } else {
        ("".to_string(), "0".to_string()) // Return empty or default values in case of incorrect input
    }
}

// Helper function to parse multiple transactions
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


fn main() {
    let user_pool = Arc::new(Mutex::new(UserPool::new()));
    let dag = Arc::new(Mutex::new(DAG::new(Arc::clone(&user_pool))));  // Initialize DAG

    // {
    //     // Initialize the user pool with some users
    //     let mut pool = user_pool.lock().unwrap();
    //     pool.add_user(User::new("Pranjal".to_string(), 1000));
    //     pool.add_user(User::new("Aditya".to_string(), 500));
    // }

    let listener = TcpListener::bind("0.0.0.0:8081").unwrap();
    println!("Server listening on port 8081");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let user_pool = Arc::clone(&user_pool);
                let dag = Arc::clone(&dag);  // Pass the DAG

                thread::spawn(move || {
                    handle_client(stream, user_pool, dag);  // Pass DAG to handle_client
                });
            }
            Err(e) => eprintln!("Failed to accept connection: {}", e),
        }
    }
}