// transaction.rs

use crate::DAGs::transaction_dag::{DAG, BlockTransaction}; // Import the DAG and Transaction
use crate::models::user::{UserPool};
use crate::models::pki::KeyPairWrapper;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use crate::models::persistence::{save_user_pool_state, save_dag_state};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingTransaction {
    pub id: String,
    pub sender: String,
    pub receiver: String,
    pub amount: u64,
    pub signature: Vec<u8>,
    pub timestamp: u64,
}

pub async fn process_transactions(
    transactions_data: Vec<(String, String, String, u64)>,
    user_pool: Arc<RwLock<UserPool>>,
    _dag: Arc<Mutex<DAG>>, // DAG will be updated upon confirmation
    stream: &mut TcpStream,
) {
    for (tx_type, sender_name, receiver_name, amount) in transactions_data {
        if tx_type != "TOKEN" {
            let _ = stream.write_all(
                format!(
                    "Error: Transaction type {} is not specified as of now\n",
                    tx_type
                )
                .as_bytes(),
            ).await;
            continue;
        }

        // Generate a unique transaction ID
        let transaction_id = Uuid::new_v4().to_string();

        // Generate timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Prepare the message to sign
        let message = format!(
            "{}:{}:{}:{}:{}",
            transaction_id, sender_name, receiver_name, amount, timestamp
        );

        // Lock user_pool once to check users and clone necessary data
        let (sender_key_pair, sender_balance) = {
            let pool = user_pool.read().await;

            // Check if sender and receiver exist
            if !pool.user_exists(&sender_name) || !pool.user_exists(&receiver_name) {
                let _ = stream.write_all(
                    format!(
                        "Error: User {} or {} does not exist\n",
                        sender_name, receiver_name
                    )
                    .as_bytes(),
                ).await;
                continue;
            }

            // Get sender's data
            let sender = pool.get_user(&sender_name).unwrap();

            // Check sender's balance
            if sender.wallet.balance < amount {
                let _ = stream.write_all(
                    format!("Error: Insufficient balance for user {}\n", sender_name).as_bytes(),
                ).await;
                continue;
            }

            // Clone key pair and balance
            (sender.key_pair_wrapper.clone(), sender.wallet.balance)
        }; // Release lock

        // Sign the message outside the lock
        let signature = match sender_key_pair.sign(message.as_bytes()) {
            Ok(sig) => sig.as_ref().to_vec(),
            Err(e) => {
                let _ = stream.write_all(
                    format!(
                        "Error: Failed to sign transaction for user {}: {}\n",
                        sender_name, e
                    )
                    .as_bytes(),
                ).await;
                continue;
            }
        };

        // Create a pending transaction
        let pending_tx = PendingTransaction {
            id: transaction_id.clone(),
            sender: sender_name.clone(),
            receiver: receiver_name.clone(),
            amount,
            signature: signature.clone(),
            timestamp,
        };

        // Lock user_pool to add the pending transaction
        {
            let mut pool = user_pool.write().await;

            // Add the pending transaction to the UserPool
            pool.pending_transactions.insert(transaction_id.clone(), pending_tx);

            // Inform the sender
            let _ = stream.write_all(
                format!(
                    "Transaction {} is pending confirmation from {}\n",
                    transaction_id, receiver_name
                )
                .as_bytes(),
            ).await;

            // Save the updated UserPool state
            if let Err(e) = save_user_pool_state(&pool) {
                eprintln!("Failed to save UserPool state: {}", e);
            }
        } // Release lock
    }
}

pub async fn finalize_transaction(
    pending_tx: PendingTransaction,
    user_pool: Arc<RwLock<UserPool>>,
    dag: Arc<Mutex<DAG>>,
) -> Result<(), String> {
    let sender_name = pending_tx.sender.clone();
    let receiver_name = pending_tx.receiver.clone();
    let amount = pending_tx.amount;
    let transaction_id = pending_tx.id.clone();
    let signature = pending_tx.signature.clone();
    let timestamp = pending_tx.timestamp;

    // Prepare the message for signature verification
    let message = format!(
        "{}:{}:{}:{}:{}",
        transaction_id, sender_name, receiver_name, amount, timestamp
    );

    // Clone sender's public key outside the lock
    let sender_public_key = {
        let pool = user_pool.read().await;
        match pool.get_user(&sender_name) {
            Some(user) => user.public_key.clone(),
            None => return Err("Sender does not exist".to_string()),
        }
    }; // Release lock

    // Perform signature verification outside the lock
    if let Err(e) = KeyPairWrapper::verify(
        &sender_public_key,
        message.as_bytes(),
        &signature,
    ) {
        return Err(format!("Error: Signature verification failed: {:?}", e));
    }

    // Lock user_pool to update balances and local DAGs
    {
        let mut pool = user_pool.write().await;

        // Verify sender's balance again
        let sender = pool.get_user_mut(&sender_name).ok_or("Sender does not exist")?;
        if sender.wallet.balance < amount {
            return Err("Error: Insufficient balance".to_string());
        }

        // Update sender's balance
        sender.wallet.balance -= amount;

        // Add transaction to sender's local DAG
        sender.local_dag.add_transaction(
            transaction_id.clone(),
            sender_name.clone(),
            receiver_name.clone(),
            amount,
            signature.clone(),
            timestamp,
        )?;

        // Update receiver's balance
        let receiver = pool.get_user_mut(&receiver_name).ok_or("Receiver does not exist")?;
        receiver.wallet.balance += amount;

        // Add transaction to receiver's local DAG
        receiver.local_dag.add_transaction(
            transaction_id.clone(),
            sender_name.clone(),
            receiver_name.clone(),
            amount,
            signature.clone(),
            timestamp,
        )?;

        // Save the updated UserPool state
        if let Err(e) = save_user_pool_state(&pool) {
            eprintln!("Failed to save UserPool state: {}", e);
        }
    } // Release lock

    // Add transaction to global DAG
    let dag_transaction = BlockTransaction::new(
        transaction_id.clone(),
        sender_name.clone(),
        receiver_name.clone(),
        amount,
        signature,
        timestamp,
    );

    {
        let mut dag = dag.lock().await;
        dag.add_transactions(vec![dag_transaction]).await?;
        // Save the updated DAG state
        if let Err(e) = save_dag_state(&dag) {
            eprintln!("Failed to save DAG state: {}", e);
        }
    }

    // Log the transaction
    {
        let pool = user_pool.read().await;
        let sender_balance = pool.get_user(&sender_name).unwrap().wallet.balance;
        let receiver_balance = pool.get_user(&receiver_name).unwrap().wallet.balance;

        println!(
            "{} sent {} tokens to {}. New balances -> {}: {}, {}: {}",
            sender_name,
            amount,
            receiver_name,
            sender_name,
            sender_balance,
            receiver_name,
            receiver_balance
        );
    }

    Ok(())
}
