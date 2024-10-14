use std::collections::HashMap;
use crate::models::pki::KeyPairWrapper; // Import the PKI module
use crate::DAGs::user_DAG::LocalDAG; // Import the local DAG
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Wallet {
    pub balance: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct User {
    pub name: String,
    pub wallet: Wallet,
    pub public_key: Vec<u8>,
    #[serde(skip)]
    pub key_pair_wrapper: KeyPairWrapper,
    pub local_dag: LocalDAG,
    #[serde(default)]
    pub initial_balance: u64, // Add this field
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
            local_dag: LocalDAG::new(),
            initial_balance, // Set initial_balance
        }
    }

    pub fn validate_local_dag(&self, user_pool: &UserPool) -> Result<(), String> {
        self.local_dag.validate_transactions(&self.name, &self.public_key, user_pool)
    }

    pub fn get_balance(&self) -> u64 {
        self.wallet.balance
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserPool {
    users: HashMap<String, User>,
}

impl UserPool {
    pub fn new() -> UserPool {
        UserPool {
            users: HashMap::new(),
        }
    }

    pub fn get_user_initial_balance(&self, name: &str) -> Option<u64> {
        self.users.get(name).map(|user| user.initial_balance)
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
