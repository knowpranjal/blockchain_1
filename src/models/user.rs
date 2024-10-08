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
    pub key_pair_wrapper: KeyPairWrapper, // Include the key pair wrapper
    pub local_dag: LocalDAG,              // Add local DAG to the user
}

impl User {
    pub fn new(name: String, initial_balance: u64) -> User {
        let key_pair_wrapper =
            KeyPairWrapper::generate().expect("Failed to generate key pair");
        let public_key = key_pair_wrapper
            .public_key()
            .expect("Failed to get public key");

        User {
            name,
            wallet: Wallet {
                balance: initial_balance,
            },
            public_key,
            key_pair_wrapper,
            local_dag: LocalDAG::new(), // Initialize the local DAG
        }
    }

    pub fn get_balance(&self) -> u64 {
        self.wallet.balance
    }
}

#[derive(Debug)]
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

    pub fn get_user(&self, name: &str) -> Option<&User> {
        self.users.get(name)
    }

    pub fn get_user_mut(&mut self, name: &str) -> Option<&mut User> {
        self.users.get_mut(name)
    }

    pub fn user_exists(&self, name: &str) -> bool {
        self.users.contains_key(name)
    }

    /// Provides read-only access to all users
    pub fn get_all_users(&self) -> &HashMap<String, User> {
        &self.users
    }

    /// Retrieves a user's public key
    pub fn get_user_public_key(&self, name: &str) -> Option<Vec<u8>> {
        self.users.get(name).map(|user| user.public_key.clone())
    }
}
