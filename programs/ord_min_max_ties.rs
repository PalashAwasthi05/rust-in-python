use std::cmp;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Player {
    score: u32,
    name: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Item {
    id: u32,
    weight: u32,
}

impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Item) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Item {
    // only the weight matters for ordering: ids break no ties
    fn cmp(&self, other: &Item) -> cmp::Ordering {
        self.weight.cmp(&other.weight)
    }
}

fn main() {
    let words = ["apple", "fig", "banana", "kiwi", "cherry", "pea"];
    println!("max_by_key len: {:?}", words.iter().max_by_key(|w| w.len()));
    println!("min_by_key len: {:?}", words.iter().min_by_key(|w| w.len()));
    println!("max_by len: {:?}", words.iter().max_by(|a, b| a.len().cmp(&b.len())));
    println!("min_by len: {:?}", words.iter().min_by(|a, b| a.len().cmp(&b.len())));
    println!("rev max_by_key: {:?}", words.iter().rev().max_by_key(|w| w.len()));
    println!("rev min_by_key: {:?}", words.iter().rev().min_by_key(|w| w.len()));

    let items = [
        Item { id: 1, weight: 5 },
        Item { id: 2, weight: 9 },
        Item { id: 3, weight: 2 },
        Item { id: 4, weight: 9 },
        Item { id: 5, weight: 2 },
    ];
    println!("max: {:?}", items.iter().max());
    println!("min: {:?}", items.iter().min());
    println!("max_by_key: {:?}", items.iter().max_by_key(|i| i.weight));
    println!("min_by_key: {:?}", items.iter().min_by_key(|i| i.weight));
    println!("cmp::max ties: {:?}", cmp::max(items[1], items[3]));
    println!("cmp::min ties: {:?}", cmp::min(items[2], items[4]));
    println!("a.max(b) ties: {:?}", items[1].max(items[3]));
    println!("a.min(b) ties: {:?}", items[2].min(items[4]));
    println!("max_by_key fn: {:?}", cmp::max_by_key(items[1], items[3], |i| i.weight));
    println!("min_by_key fn: {:?}", cmp::min_by_key(items[2], items[4], |i| i.weight));
    println!("max_by fn: {:?}", cmp::max_by(items[0], items[2], |a, b| a.id.cmp(&b.id)));
    println!("min_by fn: {:?}", cmp::min_by(items[0], items[2], |a, b| a.id.cmp(&b.id)));

    let players = vec![
        Player { score: 30, name: "ann" },
        Player { score: 50, name: "bo" },
        Player { score: 50, name: "al" },
        Player { score: 10, name: "cy" },
    ];
    let best = players.iter().max().unwrap();
    println!("best: {} {}", best.name, best.score);
    let best_by_score = players.iter().max_by_key(|p| p.score).unwrap();
    println!("best by score: {}", best_by_score.name);
    let worst = players.iter().min_by_key(|p| p.score).unwrap();
    println!("worst: {}", worst.name);
    let longest_name = players.iter().max_by_key(|p| p.name.len()).unwrap();
    println!("longest name: {}", longest_name.name);
    let shortest_name = players.iter().min_by_key(|p| p.name.len()).unwrap();
    println!("shortest name: {}", shortest_name.name);
    let p = cmp::max(players[0].clone(), players[1].clone());
    println!("cmp::max: {}", p.name);
    let idx = players.iter().enumerate().max_by_key(|(_, p)| p.score).map(|(i, _)| i);
    println!("index of max: {:?}", idx);
    let idx = players.iter().enumerate().min_by_key(|&(_, p)| cmp::Reverse(p.score)).map(|(i, _)| i);
    println!("index of min reverse: {:?}", idx);

    let empty: Vec<i32> = Vec::new();
    println!("empty max: {:?}", empty.iter().max());
    println!("empty min_by_key: {:?}", empty.iter().min_by_key(|x| **x));
    println!("cmp::max(3, 7) = {}", cmp::max(3, 7));
    println!("cmp::min(\"b\", \"a\") = {}", cmp::min("b", "a"));
    println!("max of strings: {:?}", ["pear", "Zebra", "apple"].iter().max());
    println!("max float by total_cmp: {:?}", [1.5, -2.0, 9.25, 3.0].iter().copied().max_by(f64::total_cmp));
}
