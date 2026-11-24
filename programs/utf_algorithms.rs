fn reverse(s: &str) -> String {
    s.chars().rev().collect()
}

fn is_palindrome(s: &str) -> bool {
    let cleaned: Vec<char> = s.chars().filter(|c| c.is_alphanumeric()).flat_map(|c| c.to_lowercase()).collect();
    let n = cleaned.len();
    (0..n / 2).all(|i| cleaned[i] == cleaned[n - 1 - i])
}

fn caesar(s: &str, shift: u8) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_lowercase() {
                (((c as u8 - b'a' + shift) % 26) + b'a') as char
            } else if c.is_ascii_uppercase() {
                (((c as u8 - b'A' + shift) % 26) + b'A') as char
            } else {
                c
            }
        })
        .collect()
}

fn rle(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        let mut n = 1;
        while chars.peek() == Some(&c) {
            chars.next();
            n += 1;
        }
        out.push_str(&n.to_string());
        out.push(c);
    }
    out
}

fn unrle(s: &str) -> String {
    let mut out = String::new();
    let mut count = 0usize;
    for c in s.chars() {
        if let Some(d) = c.to_digit(10) {
            count = count * 10 + d as usize;
        } else {
            out.push_str(&c.to_string().repeat(count));
            count = 0;
        }
    }
    out
}

fn main() {
    for s in ["hello", "日本語", "🦀ab", "noël"] {
        println!("{} -> {}", s, reverse(s));
    }
    for s in ["A man, a plan, a canal: Panama", "été", "Ésope reste ici et se repose", "abc"] {
        println!("{:?} palindrome={}", s, is_palindrome(s));
    }
    let enc = caesar("Hello, Wörld! xyz", 3);
    println!("{}", enc);
    println!("{}", caesar(&enc, 23));
    let r = rle("aaabccddddé🦀🦀");
    println!("{}", r);
    println!("{}", unrle(&r));
    println!("{}", unrle("12x"));
}
