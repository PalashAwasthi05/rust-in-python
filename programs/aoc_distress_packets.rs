// Day 13 2022: recursive packet parsing and ordering.
use std::cmp::Ordering;
use std::fmt;
use std::iter::Peekable;
use std::str::Chars;

const INPUT: &str = r"[1,1,3,1,1]
[1,1,5,1,1]

[[1],[2,3,4]]
[[1],4]

[9]
[[8,7,6]]

[[4,4],4,4]
[[4,4],4,4,4]

[7,7,7,7]
[7,7,7]

[]
[3]

[[[]]]
[[]]

[1,[2,[3,[4,[5,6,7]]]],8,9]
[1,[2,[3,[4,[5,6,0]]]],8,9]";

#[derive(Debug, Clone, PartialEq, Eq)]
enum Packet {
    Int(u32),
    List(Vec<Packet>),
}

impl Packet {
    fn parse(s: &str) -> Packet {
        let mut chars = s.chars().peekable();
        Self::parse_value(&mut chars)
    }

    fn parse_value(chars: &mut Peekable<Chars>) -> Packet {
        if chars.peek() == Some(&'[') {
            chars.next();
            let mut items = Vec::new();
            loop {
                match chars.peek() {
                    Some(']') => {
                        chars.next();
                        break;
                    }
                    Some(',') => {
                        chars.next();
                    }
                    _ => items.push(Self::parse_value(chars)),
                }
            }
            Packet::List(items)
        } else {
            let mut n = 0;
            while let Some(d) = chars.peek().and_then(|c| c.to_digit(10)) {
                n = n * 10 + d;
                chars.next();
            }
            Packet::Int(n)
        }
    }
}

impl Ord for Packet {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Packet::Int(a), Packet::Int(b)) => a.cmp(b),
            (Packet::List(a), Packet::List(b)) => a.cmp(b),
            (Packet::Int(a), list) => Packet::List(vec![Packet::Int(*a)]).cmp(list),
            (list, Packet::Int(b)) => list.cmp(&Packet::List(vec![Packet::Int(*b)])),
        }
    }
}

impl PartialOrd for Packet {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Packet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Packet::Int(n) => write!(f, "{}", n),
            Packet::List(items) => {
                write!(f, "[")?;
                for (i, p) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{}", p)?;
                }
                write!(f, "]")
            }
        }
    }
}

fn main() {
    let pairs: Vec<(Packet, Packet)> = INPUT
        .split("\n\n")
        .map(|block| {
            let mut it = block.lines().map(Packet::parse);
            (it.next().unwrap(), it.next().unwrap())
        })
        .collect();
    let part1: usize = pairs
        .iter()
        .enumerate()
        .filter(|(_, (a, b))| a < b)
        .map(|(i, _)| i + 1)
        .sum();
    println!("part 1: {}", part1);
    let dividers = [Packet::parse("[[2]]"), Packet::parse("[[6]]")];
    let mut all: Vec<Packet> = pairs.into_iter().flat_map(|(a, b)| vec![a, b]).collect();
    all.extend(dividers.iter().cloned());
    all.sort();
    for p in &all {
        println!("{}", p);
    }
    let key: usize = dividers
        .iter()
        .map(|d| all.binary_search(d).unwrap() + 1)
        .product();
    println!("part 2: {}", key);
}
