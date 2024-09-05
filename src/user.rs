use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Wallet {
    pub balance: u64,
}

#[derive(Debug, Clone)]
pub struct User {
    pub name: String,
    pub wallet: Wallet,
}

impl User {
    pub fn new(name: String, initial_balance: u64) -> User {
        User {
            name,
            wallet: Wallet { balance: initial_balance },
        }
    }

    pub fn send_tokens(&mut self, amount: u64, receiver: &mut User) -> Result<(), String> {
        if self.wallet.balance < amount {
            return Err(format!("{} has insufficient balance.", self.name));
        }
        self.wallet.balance -= amount;
        receiver.wallet.balance += amount;
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
