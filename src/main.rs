mod core;
mod chains;
mod models;
mod DAGs;


use libp2p::{
    identity,
    swarm::{Swarm, SwarmBuilder, SwarmEvent},
    PeerId,
};

use crate::DAGs::transaction_dag::DAG;
use crate::models::user::{User, UserPool};
use crate::models::transaction::{process_transactions, finalize_transaction};
use crate::models::pki::KeyPairWrapper;
use crate::models::persistence::{save_user_pool_state, load_user_pool_state, load_dag_state};

use tokio::sync::{Mutex, RwLock};
use std::sync::Arc;
use std::str;




use tokio::io::{AsyncBufReadExt, BufReader};
use crate::models::network::{MyBehaviour, MyRequest, MyResponse, Message, MyBehaviourEvent};
use libp2p::request_response::{Event as RequestResponseEvent, Message as RequestResponseMessage};

use libp2p::mdns::Event as MdnsEvent;

use std::collections::HashMap;
use futures::StreamExt;

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

    // Create a random key for libp2p
    let id_keys = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(id_keys.public());
    println!("Local peer id: {:?}", peer_id);

    // Set up transport
    let transport = libp2p::development_transport(id_keys.clone()).await.unwrap();

    // Create a Swarm to manage peers and events
    let behaviour = MyBehaviour::new(peer_id).expect("Failed to create MyBehaviour");
    let mut swarm = SwarmBuilder::with_tokio_executor(transport, behaviour, peer_id)
        .build();

    // Listen on all interfaces and a specific port (optional)
    swarm.listen_on("/ip4/0.0.0.0/tcp/8080".parse().unwrap()).unwrap();

    // Mapping of node names to PeerIds
    let mut peer_map: HashMap<String, PeerId> = HashMap::new();

    // Read full lines from stdin
    let mut stdin = BufReader::new(tokio::io::stdin()).lines();

    // Clone Arcs for use in event loop
    let user_pool_arc = Arc::clone(&user_pool);
    let dag_arc = Arc::clone(&dag);

    loop {
        tokio::select! {
            line = stdin.next_line() => {
                match line {
                    Ok(Some(line)) => {
                        if !line.is_empty() {
                            // Send the command to Node B specifically
                            if let Some(node_b_peer_id) = peer_map.get("NodeB") {
                                // Prepare the message
                                let message = Message::Command { command: line.clone() };
                                let message_data = serde_json::to_vec(&message).unwrap();

                                // Send the request to Node B
                                swarm.behaviour_mut().request_response.send_request(
                                    node_b_peer_id,
                                    MyRequest(message_data),
                                );
                                println!("Sent command to Node B: {}", line);
                            } else {
                                println!("Node B's PeerId not known, cannot send command to Node B.");
                            }

                            // Process the command locally
                            process_command(&line, &user_pool_arc, &dag_arc).await;
                        }
                    },
                    Ok(None) => {
                        // EOF reached
                        break;
                    },
                    Err(e) => {
                        eprintln!("Error reading line from stdin: {}", e);
                        break;
                    },
                }
            },
            event = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(event) => {
                    match event {
                        MyBehaviourEvent::RequestResponse(event) => {
                            handle_request_response_event(event, &mut swarm, &user_pool_arc, &dag_arc, &mut peer_map).await;
                        },
                        MyBehaviourEvent::Mdns(event) => {
                            handle_mdns_event(event, &mut swarm, &mut peer_map).await;
                        },
                    }
                },
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Listening on {:?}", address);
                },
                _ => {},
            }
        }
    }
}

async fn handle_request_response_event(
    event: RequestResponseEvent<MyRequest, MyResponse>,
    swarm: &mut Swarm<MyBehaviour>,
    user_pool: &Arc<RwLock<UserPool>>,
    dag: &Arc<Mutex<DAG>>,
    peer_map: &mut HashMap<String, PeerId>,
) {
    match event {
        RequestResponseEvent::Message { peer, message } => {
            match message {
                RequestResponseMessage::Request { request, channel, .. } => {
                    // Handle incoming request
                    let data = request.0;
                    if let Ok(message_str) = String::from_utf8(data) {
                        if let Ok(message) = serde_json::from_str::<Message>(&message_str) {
                            match message {
                                Message::Identity { name } => {
                                    // Store the mapping of name to PeerId
                                    peer_map.insert(name.clone(), peer);
                                    println!("Received identity from {}: {:?}", name, peer);

                                    // Send acknowledgment
                                    let response_message = Message::IdentityAck { ack: true };
                                    let response_data = serde_json::to_vec(&response_message).unwrap();
                                    swarm.behaviour_mut().request_response.send_response(
                                        channel,
                                        MyResponse(response_data),
                                    ).unwrap();
                                },
                                Message::Command { command } => {
                                    println!("Received command from {}: {}", peer, command);

                                    // Process the command
                                    process_command(&command, user_pool, dag).await;

                                    // Send acknowledgment
                                    let response_message = Message::CommandAck { ack: true };
                                    let response_data = serde_json::to_vec(&response_message).unwrap();
                                    swarm.behaviour_mut().request_response.send_response(
                                        channel,
                                        MyResponse(response_data),
                                    ).unwrap();
                                },
                                _ => {},
                            }
                        }
                    }
                },
                RequestResponseMessage::Response { response, .. } => {
                    // Handle incoming response
                    let data = response.0;
                    if let Ok(message_str) = String::from_utf8(data) {
                        if let Ok(message) = serde_json::from_str::<Message>(&message_str) {
                            match message {
                                Message::IdentityAck { ack } => {
                                    println!("Received IdentityAck from {:?}: {}", peer, ack);
                                },
                                Message::CommandAck { ack } => {
                                    println!("Received CommandAck from {:?}: {}", peer, ack);
                                },
                                _ => {},
                            }
                        }
                    }
                },
            }
        },
        RequestResponseEvent::OutboundFailure { peer, error, request_id } => {
            println!("OutboundFailure to {}: {:?}", peer, error);
        },
        RequestResponseEvent::InboundFailure { peer, error, request_id } => {
            println!("InboundFailure from {}: {:?}", peer, error);
        },
        RequestResponseEvent::ResponseSent { peer, request_id } => {
            println!("Response sent to {} for request {:?}", peer, request_id);
        },
    }
}

async fn handle_mdns_event(
    event: MdnsEvent, // Use the alias
    swarm: &mut Swarm<MyBehaviour>,
    peer_map: &mut HashMap<String, PeerId>,
) {
    match event {
        MdnsEvent::Discovered(list) => {
            for (peer_id, addr) in list {
                println!("Discovered peer {:?} at {:?}", peer_id, addr);

                // Add the peer's address
                swarm.behaviour_mut().request_response.add_address(&peer_id, addr);

                // Send our identity to the new peer
                let identity_message = Message::Identity { name: "NodeA".to_string() };
                let message_data = serde_json::to_vec(&identity_message).unwrap();
                swarm.behaviour_mut().request_response.send_request(
                    &peer_id,
                    MyRequest(message_data),
                );
            }
        },
        MdnsEvent::Expired(list) => {
            for (peer_id, addr) in list {
                println!("Expired peer {:?} at {:?}", peer_id, addr);

                // Remove the peer's address
                swarm.behaviour_mut().request_response.remove_address(&peer_id, &addr);

                // Remove from peer_map if present
                peer_map.retain(|_, &mut id| id != peer_id);
            }
        },
    }
}


async fn process_command(
    command: &str,
    user_pool: &Arc<RwLock<UserPool>>,
    dag: &Arc<Mutex<DAG>>,
) {
    // Implement command processing logic here

    println!("Processing command: {}", command);

    if command.starts_with("VIEW_PENDING_TRANSACTIONS") {
        let user_name = command.replace("VIEW_PENDING_TRANSACTIONS ", "").trim().to_string();
        let pool = user_pool.read().await;
        if !pool.user_exists(&user_name) {
            println!("Error: User does not exist");
            return;
        }
        let pending_txs: Vec<_> = pool.pending_transactions.values()
            .filter(|tx| tx.receiver == user_name)
            .cloned()
            .collect();
        drop(pool); // Release lock

        if pending_txs.is_empty() {
            println!("No pending transactions");
        } else {
            for tx in pending_txs {
                println!(
                    "Pending Transaction ID: {}, From: {}, Amount: {}",
                    tx.id, tx.sender, tx.amount
                );
            }
        }
    } else if command.starts_with("CONFIRM_TRANSACTION") {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.len() != 3 {
            println!("Usage: CONFIRM_TRANSACTION <UserName> <TransactionID>");
            return;
        }
        let user_name = parts[1];
        let transaction_id = parts[2];

        // Lock user_pool briefly to get and remove pending_tx
        let pending_tx = {
            let mut pool = user_pool.write().await;
            let pending_tx = match pool.pending_transactions.remove(transaction_id) {
                Some(tx) => tx.clone(),
                None => {
                    println!("Error: Transaction not found");
                    return;
                }
            };
            if pending_tx.receiver != user_name {
                println!("Error: Transaction is not pending for this user");
                // Re-insert pending_tx into pending_transactions
                pool.pending_transactions.insert(transaction_id.to_string(), pending_tx);
                return;
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
            Arc::clone(user_pool),
            Arc::clone(dag),
        ).await;

        match result {
            Ok(_) => {
                println!("Transaction confirmed and processed");
            }
            Err(e) => {
                println!("Error processing transaction: {}", e);
            }
        }
    } else if command.starts_with("REJECT_TRANSACTION") {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.len() != 3 {
            println!("Usage: REJECT_TRANSACTION <UserName> <TransactionID>");
            return;
        }
        let user_name = parts[1];
        let transaction_id = parts[2];
        let mut pool = user_pool.write().await;
        let pending_tx = match pool.pending_transactions.get(transaction_id) {
            Some(tx) => tx.clone(),
            None => {
                println!("Error: Transaction not found");
                return;
            }
        };
        if pending_tx.receiver != user_name {
            println!("Error: Transaction is not pending for this user");
            return;
        }

        // Remove from pending transactions
        pool.pending_transactions.remove(transaction_id);
        println!("Transaction rejected");

        // Save the updated UserPool state
        if let Err(e) = save_user_pool_state(&pool) {
            eprintln!("Failed to save UserPool state: {}", e);
        }
    } else if command.starts_with("VALIDATE_LOCAL_DAG") {
        let user_name = command.replace("VALIDATE_LOCAL_DAG ", "").trim().to_string();
        let pool = user_pool.read().await;
        if let Some(user) = pool.get_user(&user_name) {
            match user.validate_local_dag(&pool) {
                Ok(_) => {
                    println!("Local DAG is valid");
                }
                Err(e) => {
                    println!("Local DAG validation failed: {}", e);
                }
            }
        } else {
            println!("Error: User does not exist");
        }
    } else if command.starts_with("QUERY_TRANSACTION") {
        let transaction_id = command.replace("QUERY_TRANSACTION ", "").trim().to_string();
        let dag = dag.lock().await;
        if let Some(transaction) = dag.get_transaction(&transaction_id) {
            println!(
                "Transaction found: ID: {}, Sender: {}, Receiver: {}, Amount: {}, Timestamp: {}, Signature: {:?}",
                transaction.id,
                transaction.sender,
                transaction.receiver,
                transaction.amount,
                transaction.timestamp,
                transaction.signature
            );
        } else {
            println!("Transaction with ID {} not found", transaction_id);
        }
    } else if command.starts_with("FETCH_USER_DAGS") {
        let user_names_str = command.replace("FETCH_USER_DAGS ", "");
        let user_names: Vec<&str> = user_names_str.split_whitespace().collect();
        let pool = user_pool.read().await;

        for user_name in user_names {
            if let Some(user) = pool.get_user(user_name) {
                println!("User {}'s DAG:", user_name);
                println!("{}", user.local_dag.get_dag_as_string());
            } else {
                println!("Error: User {} does not exist", user_name);
            }
        }
    } else if command.starts_with("CHECK_BALANCE") {
        let user_name = command.replace("CHECK_BALANCE ", "").trim().to_string();
        let pool = user_pool.read().await;

        if let Some(user) = pool.get_user(&user_name) {
            let balance = user.get_balance();
            println!("User {} has a balance of {}", user_name, balance);
        } else {
            println!("Error: User does not exist");
        }
    } else if command.starts_with("TRANSACTION") {
        let transaction_data = command.replace("TRANSACTION ", "");
        let transactions = parse_transaction_data(&transaction_data);

        if transactions.is_empty() {
            println!("Error: Invalid transaction data");
            return;
        }

        process_transactions(
            transactions,
            Arc::clone(user_pool),
            Arc::clone(dag),
        ).await;
    } else if command.starts_with("ADD_USER") {
        let user_data = command.replace("ADD_USER ", "");
        let (name, balance_str) = parse_add_user_data(&user_data);

        let balance: u64 = match balance_str.parse() {
            Ok(val) => val,
            Err(_) => {
                println!("Error: Invalid balance");
                return;
            }
        };

        let mut pool = user_pool.write().await;
        if pool.user_exists(&name) {
            println!("Error: User already exists");
        } else {
            let user = User::new(name.clone(), balance);
            let public_key = user.public_key.clone();
            pool.add_user(user);
            println!(
                "User {} added with balance {} and public key {:?}",
                name, balance, public_key
            );
            // Save the updated UserPool state
            if let Err(e) = save_user_pool_state(&pool) {
                eprintln!("Failed to save UserPool state: {}", e);
            }
        }
    } else if command.starts_with("PRINT_DAG") {
        let dag = dag.lock().await;
        println!("Blockchain DAG:");
        for block in dag.blocks.values() {
            println!("Block ID: {}", block.id);
            println!("  Parents: {:?}", block.parent_ids);
            println!("  Children: {:?}", block.child_ids);
            println!(
                "  Transactions: {:?}",
                block.transactions.iter().map(|tx| (&tx.id, &tx.sender, &tx.receiver, tx.amount)).collect::<Vec<_>>()
            );
            println!("");
        }
    } else if command.starts_with("PRINT_USER_DAG") {
        let user_name = command.replace("PRINT_USER_DAG ", "").trim().to_string();
        let pool = user_pool.read().await;

        if let Some(user) = pool.get_user(&user_name) {
            // Call the print_dag method from LocalDAG
            user.local_dag.print_dag_in_order();
            println!("User {}'s DAG printed to the log", user_name);
        } else {
            println!("Error: User does not exist");
        }
    } else if command.starts_with("VERIFY_TRANSACTION") {
        let transaction_id = command.replace("VERIFY_TRANSACTION ", "").trim().to_string();
        let dag = dag.lock().await;
        let global_tx = match dag.get_transaction(&transaction_id) {
            Some(tx) => tx,
            None => {
                println!("Transaction with ID {} not found in global DAG", transaction_id);
                return;
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
                println!("Transaction not found in sender {}'s DAG", sender_name);
                return;
            }
        };

        // Retrieve the transaction from the receiver's local DAG
        let receiver_tx = match pool.get_user(&receiver_name)
            .and_then(|user| user.local_dag.get_transaction_by_id(&transaction_id)) {
            Some(tx) => tx,
            None => {
                println!("Transaction not found in receiver {}'s DAG", receiver_name);
                return;
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
                println!("Sender public key not found for {}", global_tx.sender);
                return;
            }
        };

        let signature_valid = KeyPairWrapper::verify(
            &sender_public_key,
            message.as_bytes(),
            &global_tx.signature,
        ).is_ok();

        if hashes_match && signature_valid {
            println!("Transaction integrity verified successfully");
        } else {
            println!("Transaction integrity verification failed");
        }
    } else {
        println!("Unknown command");
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
