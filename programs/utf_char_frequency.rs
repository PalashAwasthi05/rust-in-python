use std::collections::BTreeMap;
use std::collections::HashMap;

fn is_anagram(a: &str, b: &str) -> bool {
    let normalize = |s: &str| {
        let mut v: Vec<char> = s.chars().filter(|c| !c.is_whitespace()).flat_map(|c| c.to_lowercase()).collect();
        v.sort_unstable();
        v
    };
    normalize(a) == normalize(b)
}

fn main() {
    let text = "Ça va? Ça va très bien, merci! Größe: 日本日本 🦀🦀🦀";
    let mut freq: BTreeMap<char, usize> = BTreeMap::new();
    for c in text.chars().filter(|c| c.is_alphabetic() || *c == '🦀') {
        *freq.entry(c).or_insert(0) += 1;
    }
    for (c, n) in &freq {
        print!("{}:{} ", c, n);
    }
    println!();
    let mut by_count: Vec<(char, usize)> = freq.into_iter().collect();
    by_count.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    println!("{:?}", &by_count[..4]);
    let mut byte_len: HashMap<usize, Vec<char>> = HashMap::new();
    for c in text.chars() {
        let entry = byte_len.entry(c.len_utf8()).or_default();
        if !entry.contains(&c) {
            entry.push(c);
        }
    }
    let mut keys: Vec<_> = byte_len.keys().copied().collect();
    keys.sort();
    for k in keys {
        let chars: String = byte_len[&k].iter().take(8).collect();
        println!("{} byte(s): {}", k, chars);
    }
    let vowels = "aeiouàâéèêëîïôûùüÿ";
    let count = text.to_lowercase().chars().filter(|c| vowels.contains(*c)).count();
    println!("vowels {}", count);
    println!("{}", is_anagram("Listen", "Silent"));
    println!("{}", is_anagram("Éric", "cire"));
    println!("{}", is_anagram("Éric", "ciré"));
    let words: Vec<&str> = text.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).collect();
    let longest = words.iter().max_by_key(|w| (w.chars().count(), std::cmp::Reverse(**w))).unwrap();
    println!("{} words, longest {:?}", words.len(), longest);
    let initials: String = words.iter().filter_map(|w| w.chars().next()).collect();
    println!("{}", initials);
}
