use std::cmp::Ordering;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Card {
    rank: u8,
    suit: Suit,
}

#[derive(Debug, Eq, PartialEq)]
struct Employee {
    name: String,
    dept: String,
    salary: u32,
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

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
struct Score(f64);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
struct Version(u32, u32, u32);

fn main() {
    let mut cards = vec![
        Card { rank: 10, suit: Suit::Hearts },
        Card { rank: 2, suit: Suit::Spades },
        Card { rank: 10, suit: Suit::Clubs },
        Card { rank: 14, suit: Suit::Diamonds },
    ];
    cards.sort();
    for c in &cards {
        println!("{:?} of {:?}", c.rank, c.suit);
    }
    println!("max = {:?}", cards.iter().max().unwrap());
    println!("{}", Suit::Clubs < Suit::Spades);
    let mut staff = vec![
        Employee { name: "Zed".into(), dept: "eng".into(), salary: 100 },
        Employee { name: "Amy".into(), dept: "ops".into(), salary: 90 },
        Employee { name: "Bob".into(), dept: "eng".into(), salary: 120 },
        Employee { name: "Cat".into(), dept: "eng".into(), salary: 100 },
    ];
    staff.sort();
    for e in &staff {
        println!("{} {} {}", e.dept, e.salary, e.name);
    }
    let mut scores = vec![Score(2.5), Score(-1.0), Score(9.75), Score(3.0)];
    scores.sort_by(|a, b| b.partial_cmp(a).unwrap());
    println!("{:?}", scores);
    println!("{:?}", Score(1.0).partial_cmp(&Score(f64::NAN)));
    let versions: BTreeSet<Version> = [Version(1, 2, 3), Version(1, 10, 0), Version(0, 9, 9), Version(1, 2, 3)]
        .iter()
        .cloned()
        .collect();
    println!("{:?}", versions);
    println!("{:?}", Version(2, 0, 0).cmp(&Version(1, 99, 99)));
    println!("{:?}", Version(1, 0, 0).max(Version(1, 0, 1)));
    let newest = versions.iter().max().unwrap();
    println!("newest {:?}", newest);
    let mut words = vec!["pear", "fig", "banana", "kiwi", "apple"];
    words.sort_by(|a, b| a.len().cmp(&b.len()).then_with(|| b.cmp(a)));
    println!("{:?}", words);
    println!("{:?}", "abc".cmp("abd"));
    println!("{:?}", (1, "z").cmp(&(1, "a")));
    let clamped = Version(5, 0, 0).clamp(Version(1, 0, 0), Version(2, 0, 0));
    println!("{:?}", clamped);
    let mut by_key = staff.iter().map(|e| (&e.name, e.salary)).collect::<Vec<_>>();
    by_key.sort_by_key(|&(n, s)| (std::cmp::Reverse(s), n.clone()));
    println!("{:?}", by_key);
}
