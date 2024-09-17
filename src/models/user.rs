use std::collections::HashMap;
use crate::models::pki::KeyPairWrapper; // Import the PKI module
use crate::DAGs::user_DAG::LocalDAG; // Import the local DAG

#[derive(Debug, Clone)]
pub struct Wallet {
    pub balance: u64,
}

#[derive(Debug, Clone)]
pub struct User {
    pub name: String,
    pub wallet: Wallet,
    pub public_key: Vec<u8>,
    pub local_dag: LocalDAG, // Add local DAG to the user
}

impl User {
    pub fn new(name: String, initial_balance: u64) -> User {
        let key_pair_wrapper = KeyPairWrapper::generate().expect("Failed to generate key pair");
        let public_key = key_pair_wrapper.public_key().to_vec();

        User {
            name,
            wallet: Wallet { balance: initial_balance },
            public_key,
            local_dag: LocalDAG::new(), // Initialize the local DAG
        }
    }

    pub fn send_tokens(&mut self, amount: u64, receiver: &mut User) -> Result<(), String> {
        if self.wallet.balance < amount {
            return Err(format!("{} has insufficient balance.", self.name));
        }

        self.wallet.balance -= amount;
        receiver.wallet.balance += amount;

        // Add transaction to both sender's and receiver's local DAGs
        self.local_dag.add_transaction(self.name.clone(), receiver.name.clone(), amount)?;
        receiver.local_dag.add_transaction(self.name.clone(), receiver.name.clone(), amount)?;

        Ok(())
    }
}


pub struct UserPool {
    users: HashMap<String, User>,
}

impl UserPool {
    pub fn new() -> UserPool {
        UserPool {
            users: HashMap::new(),
        }
    }

    pub fn add_user(&mut self, user: User) {
        self.users.insert(user.name.clone(), user);
    }

    pub fn get_user(&mut self, name: &str) -> Option<&mut User> {
        self.users.get_mut(name)
    }

    pub fn user_exists(&self, name: &str) -> bool {
        self.users.contains_key(name)
    }
}
