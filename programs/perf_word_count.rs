// HashMap-heavy word counting over a generated corpus.
use std::collections::HashMap;

const WORDS: [&str; 16] = [
    "the", "quick", "brown", "fox", "jumps", "over", "lazy", "dog",
    "rust", "python", "borrow", "checker", "trait", "impl", "vector", "map",
];

fn main() {
    let mut state: u64 = 2024;
    let mut corpus = String::new();
    for i in 0..20_000 {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let index = ((state >> 33) % 16) as usize;
        if i > 0 {
            corpus.push(' ');
        }
        corpus.push_str(WORDS[index]);
    }
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for word in corpus.split_whitespace() {
        *counts.entry(word).or_insert(0) += 1;
    }
    let mut lengths: HashMap<usize, u32> = HashMap::new();
    for (word, count) in &counts {
        *lengths.entry(word.len()).or_default() += *count as u32;
    }
    let mut sorted: Vec<(&str, usize)> = counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    for (word, count) in &sorted {
        println!("{:>8} {}", word, count);
    }
    let mut by_length: Vec<_> = lengths.into_iter().collect();
    by_length.sort();
    println!("{:?}", by_length);
}
