use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Employee {
    name: String,
    dept: String,
    salary: u32,
    id: u32,
}

impl Ord for Employee {
    fn cmp(&self, other: &Self) -> Ordering {
        self.dept
            .cmp(&other.dept)
            .then_with(|| other.salary.cmp(&self.salary))
            .then(self.name.cmp(&other.name))
    }
}

impl PartialOrd for Employee {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Card {
    rank: u8,
    suit: char,
}

impl PartialOrd for Card {
    fn partial_cmp(&self, other: &Card) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Card {
    fn cmp(&self, other: &Card) -> Ordering {
        self.rank.cmp(&other.rank).reverse().then_with(|| self.suit.cmp(&other.suit))
    }
}

fn emp(name: &str, dept: &str, salary: u32, id: u32) -> Employee {
    Employee { name: name.to_string(), dept: dept.to_string(), salary, id }
}

fn main() {
    let mut staff = vec![
        emp("carol", "eng", 120, 1),
        emp("alice", "sales", 90, 2),
        emp("bob", "eng", 150, 3),
        emp("dave", "eng", 120, 4),
        emp("erin", "sales", 95, 5),
        emp("frank", "admin", 60, 6),
    ];
    staff.sort();
    for e in &staff {
        println!("{:<6} {:<5} {:>4} #{}", e.name, e.dept, e.salary, e.id);
    }
    println!("first < last: {}", staff[0] < staff[5]);
    println!("max: {}", staff.iter().max().unwrap().name);
    println!("min: {}", staff.iter().min().unwrap().name);

    let top = std::cmp::max(staff[1].clone(), staff[2].clone());
    println!("cmp::max -> {}", top.name);
    let low = std::cmp::min(staff[1].clone(), staff[2].clone());
    println!("cmp::min -> {}", low.name);

    let mut hand = vec![
        Card { rank: 10, suit: 'h' },
        Card { rank: 14, suit: 's' },
        Card { rank: 2, suit: 'c' },
        Card { rank: 14, suit: 'd' },
        Card { rank: 10, suit: 'c' },
    ];
    hand.sort();
    let shown: Vec<String> = hand.iter().map(|c| format!("{}{}", c.rank, c.suit)).collect();
    println!("{}", shown.join(" "));
    hand.sort_by(|a, b| b.cmp(a));
    let shown: Vec<String> = hand.iter().map(|c| format!("{}{}", c.rank, c.suit)).collect();
    println!("{}", shown.join(" "));
    let a = Card { rank: 3, suit: 'h' };
    let b = Card { rank: 9, suit: 'h' };
    println!("3h > 9h: {}", a > b);
    println!("max of 3h,9h: {:?}", a.max(b));
    println!("min of 3h,9h: {:?}", a.min(b));
    println!("clamp: {:?}", 15.clamp(0, 10));
    println!("ordering chain: {:?}", 1.cmp(&1).then(2.cmp(&3)).reverse());

    let mut words = vec!["banana", "Apple", "cherry", "apple", "Banana"];
    words.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()).then_with(|| b.cmp(a)));
    println!("{:?}", words);
    words.sort_by_key(|w| std::cmp::Reverse(w.len()));
    println!("{:?}", words);
}
