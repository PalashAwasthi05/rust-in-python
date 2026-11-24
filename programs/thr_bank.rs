use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug)]
struct Account {
    id: u32,
    balance: i64,
}

struct Bank {
    accounts: Vec<Mutex<Account>>,
}

impl Bank {
    fn new(n: u32, initial: i64) -> Bank {
        Bank { accounts: (0..n).map(|id| Mutex::new(Account { id, balance: initial })).collect() }
    }

    fn transfer(&self, from: usize, to: usize, amount: i64) -> Result<(), String> {
        if from == to {
            return Err(String::from("same account"));
        }
        let (first, second) = if from < to { (from, to) } else { (to, from) };
        let mut a = self.accounts[first].lock().unwrap();
        let mut b = self.accounts[second].lock().unwrap();
        let (src, dst) = if from < to { (&mut *a, &mut *b) } else { (&mut *b, &mut *a) };
        if src.balance < amount {
            return Err(format!("insufficient funds in {}", src.id));
        }
        src.balance -= amount;
        dst.balance += amount;
        Ok(())
    }

    fn total(&self) -> i64 {
        self.accounts.iter().map(|a| a.lock().unwrap().balance).sum()
    }
}

fn main() {
    let bank = Arc::new(Bank::new(4, 100));
    let plans = vec![(0, 1, 30), (1, 2, 50), (2, 3, 500), (3, 0, 10), (2, 2, 1)];
    let handles: Vec<_> = plans
        .into_iter()
        .map(|(from, to, amount)| {
            let bank = Arc::clone(&bank);
            thread::spawn(move || (from, to, amount, bank.transfer(from, to, amount)))
        })
        .collect();
    let mut outcomes: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    outcomes.sort_by_key(|o| (o.0, o.1));
    for (from, to, amount, result) in outcomes {
        match result {
            Ok(()) => println!("{} -> {} ({}) ok", from, to, amount),
            Err(e) => println!("{} -> {} ({}) failed: {}", from, to, amount, e),
        }
    }
    println!("total {}", bank.total());
    for account in bank.accounts.iter() {
        let a = account.lock().unwrap();
        println!("{:?}", *a);
    }
    let m = Mutex::new(5);
    println!("{:?}", m);
    let g = m.try_lock();
    println!("try_lock ok: {}", g.is_ok());
    drop(g);
    let guard = m.lock().unwrap();
    println!("{:?} / {} / {:?}", m, guard, guard);
}
