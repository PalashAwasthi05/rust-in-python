use std::collections::HashMap;

const TEXT: &str = "It was the best of times, it was the worst of times,
it was the age of wisdom, it was the age of foolishness,
it was the epoch of belief, it was the epoch of incredulity,
it was the season of Light, it was the season of Darkness.";

fn normalize(word: &str) -> String {
    word.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase()
}

fn main() {
    let mut counts: HashMap<String, usize> = HashMap::new();
    let mut total = 0;
    for line in TEXT.lines() {
        for raw in line.split_whitespace() {
            let w = normalize(raw);
            if w.is_empty() { continue; }
            *counts.entry(w).or_insert(0) += 1;
            total += 1;
        }
    }
    println!("total words: {}, distinct: {}", total, counts.len());

    let mut pairs: Vec<(&String, &usize)> = counts.iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));

    println!("top 6:");
    for (word, n) in pairs.iter().take(6) {
        let bar = "#".repeat(**n);
        println!("  {:<8} {:>2} {}", word, n, bar);
    }

    let singles: Vec<&str> = {
        let mut v: Vec<&str> = counts.iter().filter(|(_, &n)| n == 1).map(|(w, _)| w.as_str()).collect();
        v.sort();
        v
    };
    println!("appear once: {}", singles.join(" "));

    let mut by_len: HashMap<usize, Vec<&str>> = HashMap::new();
    for w in counts.keys() {
        by_len.entry(w.len()).or_default().push(w);
    }
    let mut lens: Vec<_> = by_len.keys().copied().collect();
    lens.sort_unstable();
    for len in lens {
        let mut ws = by_len[&len].clone();
        ws.sort();
        println!("len {}: {:?}", len, ws);
    }
    let longest = counts.keys().max_by_key(|w| (w.len(), std::cmp::Reverse((*w).clone()))).unwrap();
    println!("longest word: {}", longest);
}
