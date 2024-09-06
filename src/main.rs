mod block;
mod blockchain;
mod mainchain;
mod user;
mod transaction;


use crate::blockchain::Blockchain;
use crate::user::{User, UserPool};
use crate::transaction::process_transaction;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use std::sync::{Arc, Mutex};
use std::str;

fn handle_client(
    mut stream: TcpStream,
    blockchain: Arc<Mutex<Blockchain>>,
    user_pool: Arc<Mutex<UserPool>>,
) {
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

                    process_transaction(
                        &sender_name,
                        &receiver_name,
                        amount,
                        Arc::clone(&user_pool),
                        Arc::clone(&blockchain),
                        &mut stream
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
