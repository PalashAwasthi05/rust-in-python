#[derive(Debug)]
struct Account {
    owner: String,
    balance: i64,
}

impl Account {
    fn withdraw(&mut self, amount: i64) {
        if amount > self.balance {
            panic!(
                "insufficient funds for {}: balance {} < requested {} ({:?})",
                self.owner, self.balance, amount, self
            );
        }
        self.balance -= amount;
        println!("{} withdrew {}, left {}", self.owner, amount, self.balance);
    }
}

fn main() {
    let mut acct = Account { owner: "dora".to_string(), balance: 100 };
    acct.withdraw(30);
    acct.withdraw(60);
    let amount = 25;
    acct.withdraw(amount);
}
