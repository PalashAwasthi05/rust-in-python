use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

trait Account {
    fn id(&self) -> u32;
    fn balance(&self) -> i64;
    fn deposit(&mut self, amount: i64);
    fn withdraw(&mut self, amount: i64) -> Result<(), String> {
        if amount > self.balance() {
            return Err(format!("account {} has only {}", self.id(), self.balance()));
        }
        self.deposit(-amount);
        Ok(())
    }
    fn history(&self) -> Box<dyn Iterator<Item = i64> + '_>;
}

struct Checking {
    id: u32,
    entries: Vec<i64>,
}

struct Savings {
    id: u32,
    total: i64,
    rate_pct: i64,
}

impl Account for Checking {
    fn id(&self) -> u32 {
        self.id
    }
    fn balance(&self) -> i64 {
        self.entries.iter().sum()
    }
    fn deposit(&mut self, amount: i64) {
        self.entries.push(amount);
    }
    fn history(&self) -> Box<dyn Iterator<Item = i64> + '_> {
        Box::new(self.entries.iter().copied())
    }
}

impl Account for Savings {
    fn id(&self) -> u32 {
        self.id
    }
    fn balance(&self) -> i64 {
        self.total
    }
    fn deposit(&mut self, amount: i64) {
        self.total += amount;
    }
    fn withdraw(&mut self, amount: i64) -> Result<(), String> {
        if amount > 100 {
            Err(String::from("savings withdrawals are capped at 100"))
        } else {
            self.total -= amount;
            Ok(())
        }
    }
    fn history(&self) -> Box<dyn Iterator<Item = i64> + '_> {
        Box::new(std::iter::once(self.total))
    }
}

impl Savings {
    fn accrue(&mut self) {
        self.total += self.total * self.rate_pct / 100;
    }
}

type Shared = Rc<RefCell<dyn Account>>;

struct Bank {
    accounts: HashMap<u32, Shared>,
}

impl Bank {
    fn transfer(&self, from: u32, to: u32, amount: i64) -> Result<(), String> {
        let src = self.accounts.get(&from).ok_or("no such source")?;
        let dst = self.accounts.get(&to).ok_or_else(|| format!("no account {}", to))?;
        src.borrow_mut().withdraw(amount)?;
        dst.borrow_mut().deposit(amount);
        Ok(())
    }
    fn report(&self) -> Vec<(u32, i64)> {
        let mut v: Vec<(u32, i64)> = self.accounts.values().map(|a| (a.borrow().id(), a.borrow().balance())).collect();
        v.sort();
        v
    }
}

fn main() {
    let savings = Rc::new(RefCell::new(Savings { id: 2, total: 500, rate_pct: 10 }));
    let checking: Shared = Rc::new(RefCell::new(Checking { id: 1, entries: vec![100, 50] }));
    let mut accounts: HashMap<u32, Shared> = HashMap::new();
    accounts.insert(1, checking.clone());
    accounts.insert(2, savings.clone());
    let bank = Bank { accounts };
    println!("{:?}", bank.report());
    for (from, to, amt) in [(1, 2, 120), (1, 2, 100), (2, 1, 150), (2, 1, 80), (3, 1, 5), (1, 9, 1)] {
        match bank.transfer(from, to, amt) {
            Ok(()) => println!("moved {} from {} to {}", amt, from, to),
            Err(e) => println!("failed: {}", e),
        }
    }
    savings.borrow_mut().accrue();
    println!("{:?}", bank.report());
    let hist: Vec<i64> = checking.borrow().history().collect();
    println!("{:?}", hist);
    let biggest = checking.borrow().history().max();
    println!("{:?} rc={}", biggest, Rc::strong_count(&savings));
    let total: i64 = bank.accounts.values().map(|a| a.borrow().history().sum::<i64>()).sum();
    println!("{}", total);
}
