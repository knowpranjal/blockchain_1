mod block;
mod blockchain;
mod user;

use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::user::{User, Wallet};
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use std::sync::{Arc, Mutex};
use std::str;

fn handle_client(mut stream: TcpStream, blockchain: Arc<Mutex<Blockchain>>) {
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
                    // Parse and execute the transaction
                    let transaction_data = request.replace("TRANSACTION ", "");
                    let (sender_name, receiver_name, amount_str) = parse_transaction_data(&transaction_data);

                    let amount: u64 = match amount_str.parse() {
                        Ok(val) => val,
                        Err(_) => {
                            let _ = stream.write(b"Error: Invalid amount\n");
                            continue;
                        }
                    };

                    let mut user_a = User::new(sender_name.clone(), 1000); // Assume User A starts with 1000 tokens
                    let mut user_b = User::new(receiver_name.clone(), 500); // Assume User B starts with 500 tokens

                    match user_a.send_tokens(amount, &mut user_b) {
                        Ok(_) => {
                            println!(
                                "{} sent {} tokens to {}. New balances -> {}: {}, {}: {}",
                                sender_name, amount, receiver_name,
                                sender_name, user_a.wallet.balance,
                                receiver_name, user_b.wallet.balance
                            );

                            // Update blockchain with transaction details
                            let mut chain = blockchain.lock().unwrap();
                            let transaction_info = format!("{} sent {} tokens to {}", sender_name, amount, receiver_name);
                            chain.add_block(transaction_info.clone());

                            if let Err(e) = stream.write(b"Transaction successful\n") {
                                eprintln!("Failed to send response: {}", e);
                                return;
                            }

                            // Propagate to other node
                            // if let Err(e) = send_to_other_node("192.168.0.144:8081", &format!("TRANSACTION {}", transaction_data)) {
                            //     eprintln!("Failed to send transaction to other node: {}", e);
                            // }
                            
                        }
                        Err(e) => {
                            let _ = stream.write(format!("Error: {}\n", e).as_bytes());
                        }
                    }
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

fn send_to_other_node(address: &str, data: &str) -> std::io::Result<()> {
    println!("Attempting to connect to {}", address); // Debugging output

    let stream = TcpStream::connect(address);
    
    if let Err(e) = stream {
        eprintln!("Failed to connect to {}: {}", address, e);
        return Err(e);
    }

    let mut stream = stream.unwrap();
    println!("Connected to {}", address); // Debugging output
    stream.write_all(format!("{}\n", data).as_bytes())?;
    println!("Data sent to {}", address); // Debugging output
    Ok(())
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
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    println!("Server listening on port 8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let blockchain = Arc::clone(&blockchain);
                thread::spawn(move || {
                    handle_client(stream, blockchain);
                });
            }
            Err(e) => eprintln!("Failed to accept connection: {}", e),
        }
    }
}
