// user.rs

#[derive(Debug, Clone)]
pub struct Wallet {
    pub balance: u64,
}

impl Wallet {
    pub fn new(initial_balance: u64) -> Wallet {
        Wallet {
            balance: initial_balance,
        }
    }

    pub fn update_balance(&mut self, amount: i64) {
        // Amount can be negative for deductions or positive for additions
        self.balance = (self.balance as i64 + amount) as u64;
    }
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
            wallet: Wallet::new(initial_balance),
        }
    }

    pub fn send_tokens(&mut self, amount: u64, receiver: &mut User) -> Result<(), String> {
        if self.wallet.balance < amount {
            return Err("Insufficient funds".into());
        }

        self.wallet.update_balance(-(amount as i64));
        receiver.wallet.update_balance(amount as i64);

        Ok(())
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_creation() {
        let wallet = Wallet::new(1000);
        assert_eq!(wallet.balance, 1000);
    }

    #[test]
    fn test_wallet_update_balance() {
        let mut wallet = Wallet::new(1000);
        wallet.update_balance(500); // Adding funds
        assert_eq!(wallet.balance, 1500);

        wallet.update_balance(-200); // Deducting funds
        assert_eq!(wallet.balance, 1300);
    }

    #[test]
    fn test_user_creation() {
        let user = User::new(String::from("Alice"), 1000);
        assert_eq!(user.name, "Alice");
        assert_eq!(user.wallet.balance, 1000);
    }

    #[test]
    fn test_send_tokens_success() {
        let mut alice = User::new(String::from("Alice"), 1000);
        let mut bob = User::new(String::from("Bob"), 500);

        let result = alice.send_tokens(300, &mut bob);
        assert!(result.is_ok());
        assert_eq!(alice.wallet.balance, 700);
        assert_eq!(bob.wallet.balance, 800);
    }

    #[test]
    fn test_send_tokens_insufficient_funds() {
        let mut alice = User::new(String::from("Alice"), 100);
        let mut bob = User::new(String::from("Bob"), 500);

        let result = alice.send_tokens(200, &mut bob);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Insufficient funds");
        assert_eq!(alice.wallet.balance, 100);
        assert_eq!(bob.wallet.balance, 500);
    }

    #[test]
    fn test_send_tokens_zero_amount() {
        let mut alice = User::new(String::from("Alice"), 1000);
        let mut bob = User::new(String::from("Bob"), 500);

        let result = alice.send_tokens(0, &mut bob);
        assert!(result.is_ok());
        assert_eq!(alice.wallet.balance, 1000);
        assert_eq!(bob.wallet.balance, 500);
    }

    #[test]
    fn test_send_tokens_all_funds() {
        let mut alice = User::new(String::from("Alice"), 1000);
        let mut bob = User::new(String::from("Bob"), 500);

        let result = alice.send_tokens(1000, &mut bob);
        assert!(result.is_ok());
        assert_eq!(alice.wallet.balance, 0);
        assert_eq!(bob.wallet.balance, 1500);
    }
}
