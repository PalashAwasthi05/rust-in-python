use std::collections::HashMap;
use std::thread;

fn word_counts(text: &str) -> HashMap<String, usize> {
    let mut map = HashMap::new();
    for word in text.split_whitespace() {
        let w = word.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase();
        if !w.is_empty() {
            *map.entry(w).or_insert(0) += 1;
        }
    }
    map
}

fn main() {
    let text = "the quick brown fox jumps over the lazy dog. The dog sleeps; the fox runs! \
                A quick fox, a lazy dog, and the end.";
    let lines: Vec<String> = text.split(|c| c == '.' || c == ';' || c == '!').map(|s| s.to_string()).collect();
    let handles: Vec<thread::JoinHandle<HashMap<String, usize>>> = lines
        .into_iter()
        .map(|line| thread::spawn(move || word_counts(&line)))
        .collect();
    let mut total: HashMap<String, usize> = HashMap::new();
    for h in handles {
        for (word, n) in h.join().unwrap() {
            *total.entry(word).or_default() += n;
        }
    }
    let mut pairs: Vec<(String, usize)> = total.into_iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    for (word, n) in pairs.iter().take(6) {
        println!("{:>6}: {}", word, n);
    }

    let digits = "86967897737416471853297327050364959118205758649231";
    let chunks: Vec<String> = digits
        .chars()
        .collect::<Vec<char>>()
        .chunks(10)
        .map(|c| c.iter().collect())
        .collect();
    let mut children = vec![];
    for (i, chunk) in chunks.into_iter().enumerate() {
        children.push(thread::spawn(move || -> (usize, u32) {
            let result = chunk.chars().map(|c| c.to_digit(10).expect("digit")).sum();
            (i, result)
        }));
    }
    let intermediate: Vec<(usize, u32)> = children.into_iter().map(|c| c.join().unwrap()).collect();
    for (i, s) in &intermediate {
        println!("segment {} sum {}", i, s);
    }
    let final_result: u32 = intermediate.iter().map(|p| p.1).sum();
    println!("final sum {}", final_result);

    let primes_below = |limit: u32| -> Vec<u32> {
        (2..limit).filter(|n| (2..*n).take_while(|d| d * d <= *n).all(|d| n % d != 0)).collect()
    };
    let ranges = [(0u32, 50u32), (50, 100), (100, 150)];
    let handles: Vec<_> = ranges
        .iter()
        .map(|&(lo, hi)| thread::spawn(move || primes_below(hi).into_iter().filter(|p| *p >= lo).count()))
        .collect();
    let counts: Vec<usize> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    println!("primes per range {:?} total {}", counts, counts.iter().sum::<usize>());
}
