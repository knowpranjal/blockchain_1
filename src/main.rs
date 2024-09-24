mod core;
mod chains;
mod models;
mod DAGs;

use crate::chains::blockchain::Blockchain;
use crate::DAGs::transaction_dag::DAG;
use crate::models::user::{User, UserPool};
use crate::models::transaction::process_transactions;

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

                if request.starts_with("TRANSACTION") {
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
                            block.transactions.iter().map(|tx| (tx.id, &tx.sender, &tx.receiver, tx.amount)).collect::<Vec<_>>()
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
                } else {
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
fn parse_transaction_data(data: &str) -> Vec<(String, String, u64)> {
    let parts: Vec<&str> = data.split_whitespace().collect();
    let mut transactions = Vec::new();

    if parts.len() % 3 != 0 {
        return transactions; // Return empty if data is invalid
    }

    for chunk in parts.chunks(3) {
        let sender = chunk[0].to_string();
        let receiver = chunk[1].to_string();
        let amount = match chunk[2].parse() {
            Ok(val) => val,
            Err(_) => continue, // Skip invalid transactions
        };
        transactions.push((sender, receiver, amount));
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

    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    println!("Server listening on port 8080");

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