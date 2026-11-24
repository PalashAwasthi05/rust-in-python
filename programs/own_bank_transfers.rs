// Passing &mut through helper functions, indices instead of overlapping borrows.
#[derive(Debug)]
struct Account {
    owner: String,
    balance: i64,
    history: Vec<i64>,
}

#[derive(Debug)]
enum TxError {
    Insufficient { needed: i64, available: i64 },
    SameAccount,
    NoSuchAccount(usize),
}

struct Bank {
    accounts: Vec<Account>,
}

impl Bank {
    fn open(&mut self, owner: &str, initial: i64) -> usize {
        self.accounts.push(Account { owner: owner.to_string(), balance: initial, history: vec![initial] });
        self.accounts.len() - 1
    }

    fn get_mut(&mut self, id: usize) -> Result<&mut Account, TxError> {
        self.accounts.get_mut(id).ok_or(TxError::NoSuchAccount(id))
    }

    fn two_mut(&mut self, a: usize, b: usize) -> Result<(&mut Account, &mut Account), TxError> {
        if a == b {
            return Err(TxError::SameAccount);
        }
        if a.max(b) >= self.accounts.len() {
            return Err(TxError::NoSuchAccount(a.max(b)));
        }
        if a < b {
            let (lo, hi) = self.accounts.split_at_mut(b);
            Ok((&mut lo[a], &mut hi[0]))
        } else {
            let (lo, hi) = self.accounts.split_at_mut(a);
            Ok((&mut hi[0], &mut lo[b]))
        }
    }

    fn transfer(&mut self, from: usize, to: usize, amount: i64) -> Result<(), TxError> {
        let (src, dst) = self.two_mut(from, to)?;
        withdraw(src, amount)?;
        deposit(dst, amount);
        Ok(())
    }
}

fn withdraw(acc: &mut Account, amount: i64) -> Result<(), TxError> {
    if acc.balance < amount {
        return Err(TxError::Insufficient { needed: amount, available: acc.balance });
    }
    acc.balance -= amount;
    acc.history.push(-amount);
    Ok(())
}

fn deposit(acc: &mut Account, amount: i64) {
    acc.balance += amount;
    acc.history.push(amount);
}

fn richest(accounts: &[Account]) -> &Account {
    let mut best = &accounts[0];
    for a in accounts {
        if a.balance > best.balance {
            best = a;
        }
    }
    best
}

fn main() {
    let mut bank = Bank { accounts: Vec::new() };
    let alice = bank.open("alice", 100);
    let bob = bank.open("bob", 50);
    let carol = bank.open("carol", 0);

    println!("{:?}", bank.transfer(alice, bob, 30));
    println!("{:?}", bank.transfer(bob, carol, 70));
    println!("{:?}", bank.transfer(carol, alice, 1000));
    println!("{:?}", bank.transfer(bob, bob, 1));
    println!("{:?}", bank.transfer(bob, 9, 1));

    match bank.get_mut(carol) {
        Ok(acc) => {
            deposit(acc, 5);
            acc.owner.push_str(" (vip)");
        }
        Err(e) => println!("error {:?}", e),
    }
    if let Err(e) = bank.get_mut(42) {
        println!("error {:?}", e);
    }

    for acc in &bank.accounts {
        println!("{:<12} {:>5} {:?}", acc.owner, acc.balance, acc.history);
    }
    let r = richest(&bank.accounts);
    println!("richest: {}", r.owner);
    let total: i64 = bank.accounts.iter().map(|a| a.balance).sum();
    println!("total: {}", total);

    let ids: Vec<usize> = (0..bank.accounts.len()).collect();
    for &i in &ids {
        let interest = bank.accounts[i].balance / 10;
        let acc = &mut bank.accounts[i];
        acc.balance += interest;
    }
    let balances: Vec<i64> = bank.accounts.iter().map(|a| a.balance).collect();
    println!("{:?}", balances);
}
