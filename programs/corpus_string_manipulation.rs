fn is_palindrome(s: &str) -> bool {
    let cleaned: Vec<char> = s.chars().filter(|c| c.is_alphanumeric()).map(|c| c.to_ascii_lowercase()).collect();
    cleaned.iter().eq(cleaned.iter().rev())
}

fn caesar(s: &str, shift: i32) -> String {
    s.chars().map(|c| {
        if c.is_ascii_alphabetic() {
            let base = if c.is_ascii_uppercase() { b'A' } else { b'a' };
            let off = (c as u8 - base) as i32;
            (base + (off + shift).rem_euclid(26) as u8) as char
        } else {
            c
        }
    }).collect()
}

fn title_case(s: &str) -> String {
    s.split_whitespace().map(|w| {
        let mut cs = w.chars();
        match cs.next() {
            Some(f) => f.to_uppercase().collect::<String>() + &cs.as_str().to_lowercase(),
            None => String::new(),
        }
    }).collect::<Vec<_>>().join(" ")
}

fn snake_to_camel(s: &str) -> String {
    let mut out = String::new();
    let mut upper = false;
    for c in s.chars() {
        if c == '_' { upper = true; }
        else if upper { out.push(c.to_ascii_uppercase()); upper = false; }
        else { out.push(c); }
    }
    out
}

fn main() {
    let greeting = "  Héllo, Wörld! 👋  ";
    let t = greeting.trim();
    println!("trimmed: [{}] bytes={} chars={}", t, t.len(), t.chars().count());
    println!("upper: {} lower: {}", t.to_uppercase(), t.to_lowercase());
    println!("first char bytes: {:?}", t.chars().next().map(|c| c.len_utf8()));
    let emoji_pos = t.char_indices().find(|(_, c)| *c == '👋').map(|(i, _)| i);
    println!("emoji at byte {:?}, is boundary 2? {}", emoji_pos, t.is_char_boundary(2));
    println!("bytes of 'é': {:?}", "é".bytes().collect::<Vec<u8>>());
    println!("trim_start [{}] trim_end [{}]", greeting.trim_start().len(), greeting.trim_end().len());

    for p in ["A man, a plan, a canal: Panama", "racecar", "Rust", "No 'x' in Nixon"] {
        println!("{:?} palindrome? {}", p, is_palindrome(p));
    }
    let secret = caesar("Attack at Dawn, zebra!", 3);
    println!("caesar: {} -> {}", secret, caesar(&secret, -3));
    println!("rot13 twice: {}", caesar(&caesar("Hello", 13), 13));

    let csv = "name, age ,city\nAda,36, London\nLinus , 54,Helsinki";
    for line in csv.lines().skip(1) {
        let f: Vec<&str> = line.split(',').map(str::trim).collect();
        let age: u32 = f[1].parse().unwrap_or(0);
        println!("{} is {} in {} (next year {})", f[0], age, f[2], age + 1);
    }
    let bad: Result<i32, _> = "12a".parse::<i32>();
    println!("parse '12a': {:?}", bad.is_err());
    println!("title: {}", title_case("the qUICK brown fOX"));
    println!("camel: {}", snake_to_camel("max_retry_count"));
    let s = String::from("one two three two one");
    println!("replace: {}", s.replace("two", "2"));
    println!("find 'three': {:?}, rfind 'one': {:?}", s.find("three"), s.rfind("one"));
    println!("splitn: {:?}", s.splitn(3, ' ').collect::<Vec<_>>());
    println!("starts {} ends {} contains {}", s.starts_with("one"), s.ends_with("two"), s.contains("ee"));
    let mut built = String::with_capacity(16);
    for (i, w) in s.split(' ').enumerate() {
        if i > 0 { built.push('-'); }
        built.push_str(&w[..1]);
    }
    println!("initials: {}", built);
    let rev_words: Vec<&str> = s.split(' ').rev().collect();
    println!("rev words: {}", rev_words.join(" "));
    println!("chars rev: {}", "abc✓".chars().rev().collect::<String>());
}
