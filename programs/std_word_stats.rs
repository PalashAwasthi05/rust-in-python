// Text statistics report: word frequencies, line lengths, character classes.
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write;

const TEXT: &str = "It was the best of times, it was the worst of times,
it was the age of wisdom, it was the age of foolishness,
it was the epoch of belief, it was the epoch of incredulity,
it was the season of Light, it was the season of Darkness.";

#[derive(Debug, Default)]
struct Stats {
    lines: usize,
    words: usize,
    letters: usize,
    punctuation: usize,
    longest_word: String,
}

fn analyze(text: &str) -> Stats {
    let mut stats = Stats::default();
    for line in text.lines() {
        stats.lines += 1;
        for word in line.split_whitespace() {
            stats.words += 1;
            let clean: String = word.chars().filter(|c| c.is_alphabetic()).collect();
            if clean.len() > stats.longest_word.len() {
                stats.longest_word = clean;
            }
        }
    }
    stats.letters = text.chars().filter(char::is_ascii_alphabetic).count();
    stats.punctuation = text.chars().filter(|c| c.is_ascii_punctuation()).count();
    stats
}

fn main() {
    let stats = analyze(TEXT);
    println!("{:#?}", stats);

    let mut freq: HashMap<String, u32> = HashMap::new();
    for word in TEXT.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()) {
        *freq.entry(word.to_lowercase()).or_default() += 1;
    }
    let mut ranked: Vec<(String, u32)> = freq.into_iter().collect();
    ranked.sort_unstable_by(|(wa, ca), (wb, cb)| cb.cmp(ca).then_with(|| wa.cmp(wb)));
    let mut report = String::new();
    for (rank, (word, count)) in ranked.iter().take(6).enumerate() {
        writeln!(report, "{:>2}. {:<8} {:>3} {}", rank + 1, word, count, "#".repeat(*count as usize)).unwrap();
    }
    print!("{}", report);

    let by_len: BTreeMap<usize, Vec<&str>> = ranked.iter().fold(BTreeMap::new(), |mut acc, (w, _)| {
        acc.entry(w.len()).or_insert_with(Vec::new).push(w.as_str());
        acc
    });
    for (len, words) in by_len.iter().rev().take(3) {
        let mut words = words.clone();
        words.sort();
        println!("len {}: {}", len, words.join(", "));
    }

    let line_lengths: Vec<usize> = TEXT.lines().map(str::len).collect();
    let longest = line_lengths.iter().enumerate().max_by_key(|&(_, len)| len).map(|(i, _)| i);
    println!("line lengths {:?}, longest line index {:?}", line_lengths, longest);
    let avg = line_lengths.iter().sum::<usize>() as f64 / line_lengths.len() as f64;
    println!("average line length {:.2}", avg);

    let first_words: HashSet<&str> = TEXT.lines().filter_map(|l| l.split_whitespace().next()).collect();
    let mut fw: Vec<_> = first_words.into_iter().collect();
    fw.sort();
    println!("first words {:?}", fw);
    let capitals: String = TEXT.chars().filter(|c| c.is_uppercase()).collect();
    println!("capitals {:?}", capitals);
    let vowels = TEXT.chars().filter(|c| matches!(c.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u')).count();
    println!("vowels {} ratio {:.3}", vowels, vowels as f64 / stats.letters as f64);
    let acronym: String = TEXT.lines().next().unwrap().split_whitespace().map(|w| w.chars().next().unwrap().to_ascii_uppercase()).collect();
    println!("acronym {}", acronym);
    let reversed_words: Vec<String> = "hello brave new world".split(' ').rev().map(|w| w.chars().rev().collect()).collect();
    println!("{}", reversed_words.join(" "));
    let centered = format!("[{:^30}]", "a tale of two cities");
    println!("{}", centered);
    let wrapped = TEXT.split_whitespace().fold(vec![String::new()], |mut lines, word| {
        let last = lines.last_mut().unwrap();
        if last.len() + word.len() + 1 > 40 && !last.is_empty() {
            lines.push(word.to_string());
        } else {
            if !last.is_empty() {
                last.push(' ');
            }
            last.push_str(word);
        }
        lines
    });
    for line in &wrapped {
        println!("|{:<40}|", line);
    }
}
