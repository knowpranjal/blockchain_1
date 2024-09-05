mod block;
mod blockchain;
mod user;

use crate::blockchain::Blockchain;
use crate::user::{User, UserPool};
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use std::sync::{Arc, Mutex};
use std::str;

fn handle_client(mut stream: TcpStream, blockchain: Arc<Mutex<Blockchain>>, user_pool: Arc<Mutex<UserPool>>) {
    let mut buffer = [0; 512];
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

                if request.starts_with("ADD_BLOCK") {
                    let data = request.replace("ADD_BLOCK ", "");
                    let mut chain = blockchain.lock().unwrap();
                    chain.add_block(data.clone());
                    println!("Block added: {:?}", chain.blocks.last().unwrap());
                    
                    if let Err(e) = stream.write(b"Block added\n") {
                        eprintln!("Failed to send response: {}", e);
                        return;
                    }
                    
                    // Propagate to other node
                    // if let Err(e) = send_to_other_node("192.168.0.144:8081", &format!("ADD_BLOCK {}", data)) {
                    //     eprintln!("Failed to send block to other node: {}", e);
                    // }

                } else if request.starts_with("TRANSACTION") {
                    let transaction_data = request.replace("TRANSACTION ", "");
                    let (sender_name, receiver_name, amount_str) = parse_transaction_data(&transaction_data);

                    let amount: u64 = match amount_str.parse() {
                        Ok(val) => val,
                        Err(_) => {
                            let _ = stream.write(b"Error: Invalid amount\n");
                            continue;
                        }
                    };

                    // Lock the user pool to check if users exist
                    let mut pool = user_pool.lock().unwrap();

                    if !pool.user_exists(&sender_name) || !pool.user_exists(&receiver_name) {
                        let _ = stream.write(b"Error: One or both users do not exist\n");
                        continue;
                    }

                    let sender_new_balance;
                    let receiver_new_balance;

                    // First, get the sender and perform necessary checks
                    {
                        let sender = pool.get_user(&sender_name).unwrap();
                        if sender.wallet.balance < amount {
                            let _ = stream.write(b"Error: Insufficient balance\n");
                            continue;
                        }

                        // Deduct the amount from the sender and store the new balance
                        sender.wallet.balance -= amount;
                        sender_new_balance = sender.wallet.balance;
                    } // Release the mutable borrow of `sender`

                    // Now, get the receiver and add the amount
                    {
                        let receiver = pool.get_user(&receiver_name).unwrap();
                        receiver.wallet.balance += amount;
                        receiver_new_balance = receiver.wallet.balance;
                    } // Release the mutable borrow of `receiver`

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
                } else if request.starts_with("PRINT_CHAIN") {
                    let chain = blockchain.lock().unwrap();
                    let mut response = String::new();
                    for block in &chain.blocks {
                        response.push_str(&format!("{:?}\n", block));
                    }
                    if let Err(e) = stream.write(response.as_bytes()) {
                        eprintln!("Failed to send response: {}", e);
                        return;
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



fn parse_transaction_data(data: &str) -> (String, String, String) {
    let parts: Vec<&str> = data.split_whitespace().collect();
    if parts.len() == 3 {
        (parts[0].to_string(), parts[1].to_string(), parts[2].to_string())
    } else {
        ("".to_string(), "".to_string(), "0".to_string()) // Return empty or default values in case of incorrect input
    }
}

fn main() {
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));
    let user_pool = Arc::new(Mutex::new(UserPool::new()));

    {
        // Initialize the user pool with some users
        let mut pool = user_pool.lock().unwrap();
        pool.add_user(User::new("Pranjal".to_string(), 1000));
        pool.add_user(User::new("Aditya".to_string(), 500));
    }

    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    println!("Server listening on port 8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let blockchain = Arc::clone(&blockchain);
                let user_pool = Arc::clone(&user_pool);
                thread::spawn(move || {
                    handle_client(stream, blockchain, user_pool);
                });
            }
            Err(e) => eprintln!("Failed to accept connection: {}", e),
        }
    }
}
