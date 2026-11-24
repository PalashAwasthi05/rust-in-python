// Field visibility: pub, pub(crate), pub(super), private, with constructors.
mod bank {
    #[derive(Debug)]
    pub struct Account {
        pub owner: String,
        pub(crate) id: u32,
        balance: i64,
    }

    impl Account {
        pub fn new(owner: &str, id: u32) -> Self {
            Account { owner: owner.to_string(), id, balance: 0 }
        }

        pub fn deposit(&mut self, amount: i64) -> &mut Self {
            self.balance += amount;
            self
        }

        pub fn balance(&self) -> i64 {
            self.balance
        }
    }

    pub mod audit {
        pub struct Entry {
            pub(super) amount: i64,
            pub(in crate::bank) note: &'static str,
        }

        pub fn make(amount: i64) -> Entry {
            Entry { amount, note: "auto" }
        }

        pub fn read(entry: &Entry) -> String {
            format!("{}:{}", entry.note, entry.amount)
        }
    }

    pub fn inspect(entry: &audit::Entry) -> i64 {
        // pub(super) field visible in the parent module
        entry.amount * 2
    }

    pub fn note_of(entry: &audit::Entry) -> &'static str {
        entry.note
    }

    pub struct Token(pub u8, u8);

    impl Token {
        pub fn new(a: u8) -> Token {
            Token(a, a.wrapping_mul(3))
        }
        pub fn secret(&self) -> u8 {
            self.1
        }
    }
}

fn main() {
    let mut acct = bank::Account::new("ada", 7);
    acct.deposit(50).deposit(25);
    acct.owner.push_str(" lovelace");
    acct.id += 1;
    println!("{} #{} has {}", acct.owner, acct.id, acct.balance());
    println!("{:?}", acct);

    let e = bank::audit::make(21);
    println!("{} {} {}", bank::inspect(&e), bank::note_of(&e), bank::audit::read(&e));

    let t = bank::Token::new(5);
    println!("token {} {}", t.0, t.secret());
}
