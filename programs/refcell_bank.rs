use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

#[derive(Debug)]
struct Account { owner: String, balance: i64, log: Vec<String> }

type Shared = Rc<RefCell<Account>>;

struct Bank { accounts: BTreeMap<u32, Shared>, next_id: u32 }

impl Bank {
    fn new() -> Self { Bank { accounts: BTreeMap::new(), next_id: 100 } }

    fn open(&mut self, owner: &str, initial: i64) -> (u32, Shared) {
        let acct = Rc::new(RefCell::new(Account { owner: owner.into(), balance: initial, log: vec![format!("open {}", initial)] }));
        let id = self.next_id;
        self.next_id += 1;
        self.accounts.insert(id, Rc::clone(&acct));
        (id, acct)
    }

    fn transfer(&self, from: u32, to: u32, amount: i64) -> Result<(), String> {
        if from == to { return Err("same account".into()); }
        let src = self.accounts.get(&from).ok_or(format!("no account {}", from))?;
        let dst = self.accounts.get(&to).ok_or(format!("no account {}", to))?;
        {
            let s = src.borrow();
            if s.balance < amount {
                return Err(format!("{} has only {}", s.owner, s.balance));
            }
        }
        let mut s = src.borrow_mut();
        let mut d = dst.borrow_mut();
        s.balance -= amount;
        d.balance += amount;
        s.log.push(format!("sent {} to {}", amount, d.owner));
        d.log.push(format!("got {} from {}", amount, s.owner));
        Ok(())
    }

    fn apply_interest(&self, pct: i64) {
        for acct in self.accounts.values() {
            let mut a = acct.borrow_mut();
            let interest = a.balance * pct / 100;
            a.balance += interest;
            a.log.push(format!("interest {}", interest));
        }
    }

    fn total(&self) -> i64 { self.accounts.values().map(|a| a.borrow().balance).sum() }
}

fn main() {
    let mut bank = Bank::new();
    let (alice_id, alice) = bank.open("alice", 500);
    let (bob_id, _) = bank.open("bob", 120);
    let (carol_id, carol) = bank.open("carol", 0);
    println!("rc count alice: {}", Rc::strong_count(&alice));

    let moves = [(alice_id, bob_id, 200), (bob_id, carol_id, 400), (bob_id, carol_id, 300), (carol_id, carol_id, 1), (alice_id, 999, 5)];
    for (f, t, amt) in moves {
        match bank.transfer(f, t, amt) {
            Ok(()) => println!("{} -> {} : {} ok", f, t, amt),
            Err(e) => println!("{} -> {} : {} failed ({})", f, t, amt, e),
        }
    }
    carol.borrow_mut().balance += 50;
    bank.apply_interest(10);
    for (id, acct) in &bank.accounts {
        let a = acct.borrow();
        println!("{} {:<6} {:>5}  {:?}", id, a.owner, a.balance, a.log);
    }
    println!("total {}", bank.total());
    println!("alice via handle: {}", alice.borrow().balance);
    let can_borrow = alice.try_borrow_mut().is_ok();
    let guard = alice.borrow();
    println!("try_borrow_mut free={} while borrowed={}", can_borrow, alice.try_borrow_mut().is_err());
    drop(guard);
    drop(bank);
    println!("after bank dropped, rc count alice: {}", Rc::strong_count(&alice));
}
