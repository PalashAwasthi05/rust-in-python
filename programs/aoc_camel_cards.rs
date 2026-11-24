// Day 7 2023: poker hands with custom ordering, jokers, and derived Ord on an enum.
use std::cmp::Ordering;
use std::collections::HashMap;

const INPUT: &str = r"32T3K 765
T55J5 684
KK677 28
KTJJT 220
QQQJA 483
JJJJJ 1
2345J 77
AAKKQ 12";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Kind {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    FullHouse,
    FourOfAKind,
    FiveOfAKind,
}

#[derive(Debug, PartialEq, Eq)]
struct Hand {
    kind: Kind,
    cards: [u8; 5],
    bid: u64,
}

impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Hand {
    fn cmp(&self, other: &Self) -> Ordering {
        self.kind.cmp(&other.kind).then(self.cards.cmp(&other.cards))
    }
}

fn card_value(c: char, jokers: bool) -> u8 {
    match c {
        'A' => 14,
        'K' => 13,
        'Q' => 12,
        'J' => if jokers { 1 } else { 11 },
        'T' => 10,
        d => d.to_digit(10).unwrap() as u8,
    }
}

fn classify(cards: &[u8; 5]) -> Kind {
    let mut counts: HashMap<u8, usize> = HashMap::new();
    let mut jokers = 0;
    for &c in cards {
        if c == 1 {
            jokers += 1;
        } else {
            *counts.entry(c).or_insert(0) += 1;
        }
    }
    let mut groups: Vec<usize> = counts.into_values().collect();
    groups.sort_unstable_by(|a, b| b.cmp(a));
    if groups.is_empty() {
        groups.push(0);
    }
    groups[0] += jokers;
    match groups.as_slice() {
        [5] => Kind::FiveOfAKind,
        [4, ..] => Kind::FourOfAKind,
        [3, 2] => Kind::FullHouse,
        [3, ..] => Kind::ThreeOfAKind,
        [2, 2, ..] => Kind::TwoPair,
        [2, ..] => Kind::OnePair,
        _ => Kind::HighCard,
    }
}

fn winnings(jokers: bool) -> u64 {
    let mut hands: Vec<Hand> = INPUT
        .lines()
        .map(|line| {
            let (cards, bid) = line.split_once(' ').unwrap();
            let mut arr = [0u8; 5];
            for (slot, c) in arr.iter_mut().zip(cards.chars()) {
                *slot = card_value(c, jokers);
            }
            Hand { kind: classify(&arr), cards: arr, bid: bid.parse().unwrap() }
        })
        .collect();
    hands.sort();
    for h in &hands {
        println!("  {:?} {:?} {}", h.kind, h.cards, h.bid);
    }
    hands.iter().enumerate().map(|(rank, h)| (rank as u64 + 1) * h.bid).sum()
}

fn main() {
    println!("part 1: {}", winnings(false));
    println!("part 2: {}", winnings(true));
    let mut kinds = vec![Kind::FullHouse, Kind::HighCard, Kind::FiveOfAKind, Kind::TwoPair];
    kinds.sort_by(|a, b| b.cmp(a));
    println!("{:?}", kinds);
    println!("{}", Kind::OnePair < Kind::TwoPair && Kind::FullHouse.max(Kind::ThreeOfAKind) == Kind::FullHouse);
}
