use std::collections::{BTreeMap, HashMap, VecDeque};
use std::fmt;

type AccountId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Kind {
    Checking,
    Savings,
    Credit,
}

#[derive(Debug, Clone, PartialEq)]
enum TxError {
    NoSuchAccount(AccountId),
    Insufficient { id: AccountId, balance: i64, wanted: i64 },
    Frozen(AccountId),
    SameAccount,
    InvalidAmount(i64),
}

impl fmt::Display for TxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TxError::NoSuchAccount(id) => write!(f, "account #{} does not exist", id),
            TxError::Insufficient { id, balance, wanted } => write!(
                f,
                "account #{} has {} but {} was requested",
                id,
                Money(*balance),
                Money(*wanted)
            ),
            TxError::Frozen(id) => write!(f, "account #{} is frozen", id),
            TxError::SameAccount => write!(f, "cannot transfer to the same account"),
            TxError::InvalidAmount(a) => write!(f, "invalid amount {}", a),
        }
    }
}

/// Cents, displayed as dollars.
#[derive(Clone, Copy)]
struct Money(i64);

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let sign = if self.0 < 0 { "-" } else { "" };
        let abs = self.0.abs();
        let s = format!("{}${}.{:02}", sign, abs / 100, abs % 100);
        f.pad(&s)
    }
}

trait Account: fmt::Debug {
    fn id(&self) -> AccountId;
    fn owner(&self) -> &str;
    fn balance(&self) -> i64;
    fn kind(&self) -> Kind;
    fn adjust(&mut self, delta: i64);
    fn limit(&self) -> i64 {
        0
    }
    fn month_end(&mut self) -> Option<i64> {
        None
    }
    fn available(&self) -> i64 {
        self.balance() + self.limit()
    }
    fn describe(&self) -> String {
        format!("#{:<3} {:<8} {:<9} {:>10}", self.id(), self.owner(), format!("{:?}", self.kind()), Money(self.balance()))
    }
}

#[derive(Debug)]
struct Checking {
    id: AccountId,
    owner: String,
    cents: i64,
    overdraft: i64,
}

#[derive(Debug)]
struct Savings {
    id: AccountId,
    owner: String,
    cents: i64,
    rate_bp: i64,
}

#[derive(Debug)]
struct Credit {
    id: AccountId,
    owner: String,
    cents: i64,
    line: i64,
}

macro_rules! basic_account {
    ($t:ty, $kind:expr) => {
        fn id(&self) -> AccountId { self.id }
        fn owner(&self) -> &str { &self.owner }
        fn balance(&self) -> i64 { self.cents }
        fn kind(&self) -> Kind { $kind }
        fn adjust(&mut self, delta: i64) { self.cents += delta; }
    };
}

impl Account for Checking {
    basic_account!(Checking, Kind::Checking);
    fn limit(&self) -> i64 {
        self.overdraft
    }
    fn month_end(&mut self) -> Option<i64> {
        if self.cents < 0 {
            let fee = 1500;
            self.cents -= fee;
            Some(-fee)
        } else {
            None
        }
    }
}

impl Account for Savings {
    basic_account!(Savings, Kind::Savings);
    fn month_end(&mut self) -> Option<i64> {
        let interest = self.cents * self.rate_bp / 10_000 / 12;
        self.cents += interest;
        if interest > 0 { Some(interest) } else { None }
    }
}

impl Account for Credit {
    basic_account!(Credit, Kind::Credit);
    fn limit(&self) -> i64 {
        self.line
    }
    fn month_end(&mut self) -> Option<i64> {
        if self.cents >= 0 {
            return None;
        }
        let charge = (-self.cents) * 2 / 100;
        self.cents -= charge;
        Some(-charge)
    }
}

#[derive(Debug, Clone)]
enum Tx {
    Deposit(AccountId, i64),
    Withdraw(AccountId, i64),
    Transfer { from: AccountId, to: AccountId, amount: i64 },
    Freeze(AccountId),
    Thaw(AccountId),
    MonthEnd,
}

struct Bank {
    accounts: BTreeMap<AccountId, Box<dyn Account>>,
    frozen: Vec<AccountId>,
    ledger: Vec<(usize, String)>,
    recent: VecDeque<String>,
    next_id: AccountId,
}

impl Bank {
    fn new() -> Self {
        Bank { accounts: BTreeMap::new(), frozen: vec![], ledger: vec![], recent: VecDeque::new(), next_id: 100 }
    }

    fn open(&mut self, owner: &str, kind: Kind, initial: i64) -> AccountId {
        let id = self.next_id;
        self.next_id += 1;
        let owner = owner.to_owned();
        let acct: Box<dyn Account> = match kind {
            Kind::Checking => Box::new(Checking { id, owner, cents: initial, overdraft: 5_000 }),
            Kind::Savings => Box::new(Savings { id, owner, cents: initial, rate_bp: 240 }),
            Kind::Credit => Box::new(Credit { id, owner, cents: initial, line: 100_000 }),
        };
        self.accounts.insert(id, acct);
        id
    }

    fn get_mut(&mut self, id: AccountId) -> Result<&mut Box<dyn Account>, TxError> {
        if self.frozen.contains(&id) {
            return Err(TxError::Frozen(id));
        }
        self.accounts.get_mut(&id).ok_or(TxError::NoSuchAccount(id))
    }

    fn withdraw(&mut self, id: AccountId, amount: i64) -> Result<(), TxError> {
        let acct = self.get_mut(id)?;
        if acct.available() < amount {
            return Err(TxError::Insufficient { id, balance: acct.balance(), wanted: amount });
        }
        acct.adjust(-amount);
        Ok(())
    }

    fn apply(&mut self, tx: &Tx) -> Result<String, TxError> {
        match *tx {
            Tx::Deposit(_, amt) | Tx::Withdraw(_, amt) | Tx::Transfer { amount: amt, .. } if amt <= 0 => {
                Err(TxError::InvalidAmount(amt))
            }
            Tx::Deposit(id, amt) => {
                self.get_mut(id)?.adjust(amt);
                Ok(format!("deposited {} to #{}", Money(amt), id))
            }
            Tx::Withdraw(id, amt) => {
                self.withdraw(id, amt)?;
                Ok(format!("withdrew {} from #{}", Money(amt), id))
            }
            Tx::Transfer { from, to, amount } => {
                if from == to {
                    return Err(TxError::SameAccount);
                }
                self.get_mut(to)?;
                self.withdraw(from, amount)?;
                self.get_mut(to)?.adjust(amount);
                Ok(format!("moved {} #{} -> #{}", Money(amount), from, to))
            }
            Tx::Freeze(id) => {
                self.get_mut(id)?;
                self.frozen.push(id);
                Ok(format!("froze #{}", id))
            }
            Tx::Thaw(id) => {
                let before = self.frozen.len();
                self.frozen.retain(|&f| f != id);
                Ok(if self.frozen.len() < before { format!("thawed #{}", id) } else { format!("#{} was not frozen", id) })
            }
            Tx::MonthEnd => {
                let mut notes = Vec::new();
                for (id, acct) in self.accounts.iter_mut() {
                    if let Some(delta) = acct.month_end() {
                        notes.push(format!("#{} {}", id, Money(delta)));
                    }
                }
                Ok(format!("month end: {}", if notes.is_empty() { "nothing".to_string() } else { notes.join(", ") }))
            }
        }
    }

    fn process(&mut self, txs: &[Tx]) {
        for (n, tx) in txs.iter().enumerate() {
            let line = match self.apply(tx) {
                Ok(msg) => msg,
                Err(e) => format!("REJECTED {:?}: {}", tx, e),
            };
            self.ledger.push((n, line.clone()));
            self.recent.push_back(line);
            if self.recent.len() > 3 {
                self.recent.pop_front();
            }
        }
    }

    fn report(&self) {
        println!("{:-^40}", " accounts ");
        for acct in self.accounts.values() {
            let flag = if self.frozen.contains(&acct.id()) { " (frozen)" } else { "" };
            println!("{}{}", acct.describe(), flag);
        }
        let mut by_owner: HashMap<&str, i64> = HashMap::new();
        let mut by_kind: BTreeMap<Kind, (usize, i64)> = BTreeMap::new();
        for acct in self.accounts.values() {
            *by_owner.entry(acct.owner()).or_default() += acct.balance();
            let e = by_kind.entry(acct.kind()).or_insert((0, 0));
            e.0 += 1;
            e.1 += acct.balance();
        }
        let mut owners: Vec<_> = by_owner.into_iter().collect();
        owners.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
        println!("{:-^40}", " net worth ");
        for (owner, total) in &owners {
            println!("{:<10}{:>12}", owner, Money(*total));
        }
        println!("{:-^40}", " by kind ");
        for (kind, (count, total)) in &by_kind {
            println!("{:<10}{:>3} {:>12}", format!("{:?}", kind), count, Money(*total));
        }
        let total: i64 = self.accounts.values().map(|a| a.balance()).sum();
        println!("bank total {}", Money(total));
    }
}

fn main() {
    let mut bank = Bank::new();
    let alice = bank.open("alice", Kind::Checking, 120_00);
    let alice_s = bank.open("alice", Kind::Savings, 5_000_00);
    let bob = bank.open("bob", Kind::Checking, 30_00);
    let bob_c = bank.open("bob", Kind::Credit, 0);
    let carol = bank.open("carol", Kind::Savings, 750_00);

    let txs = vec![
        Tx::Deposit(alice, 25_50),
        Tx::Withdraw(bob, 70_00),
        Tx::Withdraw(bob, 20_00),
        Tx::Transfer { from: alice_s, to: carol, amount: 1_234_56 },
        Tx::Transfer { from: bob_c, to: bob, amount: 400_00 },
        Tx::Transfer { from: carol, to: carol, amount: 1 },
        Tx::Deposit(999, 10),
        Tx::Withdraw(alice, -5),
        Tx::Freeze(carol),
        Tx::Deposit(carol, 100),
        Tx::Transfer { from: alice, to: carol, amount: 100 },
        Tx::Thaw(carol),
        Tx::Thaw(carol),
        Tx::Withdraw(bob_c, 2_000_00),
        Tx::Withdraw(bob, 450_00),
        Tx::MonthEnd,
        Tx::MonthEnd,
    ];
    bank.process(&txs);
    for (n, line) in &bank.ledger {
        println!("{:>2}: {}", n, line);
    }
    println!("recent: {:?}", bank.recent);
    bank.report();
    println!("{:>12}|{:<12}|{:^12}|", Money(-123_45), Money(5), Money(1_000_000_00));
    let richest = bank.accounts.values().max_by_key(|a| a.balance()).map(|a| a.owner().to_string());
    println!("richest single account: {:?}", richest);
}
