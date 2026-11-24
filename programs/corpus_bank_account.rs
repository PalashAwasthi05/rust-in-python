use std::fmt;

#[derive(Debug)]
enum TxError {
    InsufficientFunds { needed: u64, available: u64 },
    AccountFrozen,
    ZeroAmount,
}

impl fmt::Display for TxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TxError::InsufficientFunds { needed, available } => {
                write!(f, "insufficient funds: needed {}, available {}", needed, available)
            }
            TxError::AccountFrozen => write!(f, "account is frozen"),
            TxError::ZeroAmount => write!(f, "amount must be positive"),
        }
    }
}

struct Account {
    owner: String,
    balance: u64,
    frozen: bool,
    history: Vec<i64>,
}

impl Account {
    fn new(owner: &str, initial: u64) -> Self {
        Account { owner: owner.to_string(), balance: initial, frozen: false, history: vec![initial as i64] }
    }

    fn deposit(&mut self, amount: u64) -> Result<u64, TxError> {
        if self.frozen { return Err(TxError::AccountFrozen); }
        if amount == 0 { return Err(TxError::ZeroAmount); }
        self.balance += amount;
        self.history.push(amount as i64);
        Ok(self.balance)
    }

    fn withdraw(&mut self, amount: u64) -> Result<u64, TxError> {
        if self.frozen { return Err(TxError::AccountFrozen); }
        if amount == 0 { return Err(TxError::ZeroAmount); }
        if amount > self.balance {
            return Err(TxError::InsufficientFunds { needed: amount, available: self.balance });
        }
        self.balance -= amount;
        self.history.push(-(amount as i64));
        Ok(self.balance)
    }
}

fn transfer(from: &mut Account, to: &mut Account, amount: u64) -> Result<(), TxError> {
    from.withdraw(amount)?;
    to.deposit(amount)?;
    Ok(())
}

fn main() {
    let mut alice = Account::new("Alice", 100);
    let mut bob = Account::new("Bob", 50);

    match alice.deposit(25) {
        Ok(b) => println!("{} deposited, balance {}", alice.owner, b),
        Err(e) => println!("error: {}", e),
    }
    if let Err(e) = bob.withdraw(80) {
        println!("{} withdraw failed: {}", bob.owner, e);
    }
    if let Err(e) = alice.deposit(0) {
        println!("deposit failed: {}", e);
    }
    transfer(&mut alice, &mut bob, 60).unwrap();
    println!("after transfer: alice={} bob={}", alice.balance, bob.balance);

    bob.frozen = true;
    match transfer(&mut alice, &mut bob, 10) {
        Ok(()) => println!("transfer ok"),
        Err(e) => println!("transfer failed: {}", e),
    }
    println!("alice={} bob={}", alice.balance, bob.balance);
    println!("alice history: {:?}", alice.history);
    let net: i64 = bob.history.iter().sum();
    println!("bob net: {}", net);
}
