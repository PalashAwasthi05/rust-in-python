use std::collections::BTreeMap;

fn rle_encode(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        let mut n = 1;
        while chars.next_if_eq(&c).is_some() { n += 1; }
        if n > 1 { out.push_str(&n.to_string()); }
        out.push(c);
    }
    out
}

fn rle_decode(s: &str) -> String {
    let mut out = String::new();
    let mut n = 0usize;
    for c in s.chars() {
        if let Some(d) = c.to_digit(10) {
            n = n * 10 + d as usize;
        } else {
            out.extend(std::iter::repeat(c).take(n.max(1)));
            n = 0;
        }
    }
    out
}

fn group_anagrams<'a>(words: &[&'a str]) -> BTreeMap<String, Vec<&'a str>> {
    let mut groups: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    for w in words {
        let mut key: Vec<char> = w.to_lowercase().chars().collect();
        key.sort_unstable();
        groups.entry(key.into_iter().collect()).or_default().push(w);
    }
    groups
}

fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cur = String::new();
    for word in text.split_whitespace() {
        if !cur.is_empty() && cur.len() + 1 + word.len() > width {
            lines.push(std::mem::take(&mut cur));
        }
        if !cur.is_empty() { cur.push(' '); }
        cur.push_str(word);
    }
    if !cur.is_empty() { lines.push(cur); }
    lines
}

fn longest_common_prefix(words: &[&str]) -> String {
    let first = match words.first() { Some(f) => *f, None => return String::new() };
    let mut end = first.len();
    for w in &words[1..] {
        end = end.min(first.bytes().zip(w.bytes()).take_while(|(a, b)| a == b).count());
    }
    first[..end].to_string()
}

fn main() {
    for s in ["aaabccddddde", "abc", "zzzzzzzzzzzz", ""] {
        let e = rle_encode(s);
        println!("{:?} -> {:?} -> roundtrip ok: {}", s, e, rle_decode(&e) == s);
    }
    let words = ["listen", "Silent", "enlist", "google", "gooegl", "cat", "act", "tac", "dog"];
    for (key, group) in group_anagrams(&words) {
        println!("{:<7} {:?}", key, group);
    }
    let text = "The quick brown fox jumps over the lazy dog and keeps running far beyond the hills";
    for w in [20, 32] {
        println!("width {}:", w);
        for line in wrap(text, w) {
            println!("|{:<width$}|", line, width = w);
        }
    }
    println!("lcp: {:?}", longest_common_prefix(&["interstellar", "internet", "interval"]));
    println!("lcp none: {:?}", longest_common_prefix(&["abc", "xyz"]));
    let vowels = text.chars().filter(|c| "aeiouAEIOU".contains(*c)).count();
    let caps: String = text.split(' ').filter_map(|w| w.chars().next()).filter(|c| c.is_uppercase()).collect();
    println!("vowels {} caps {:?}", vowels, caps);
}
