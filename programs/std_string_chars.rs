// Character-level string processing.
fn caesar(text: &str, shift: u8) -> String {
    text.chars()
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

fn is_palindrome(s: &str) -> bool {
    let cleaned: Vec<char> = s.chars().filter(|c| c.is_alphanumeric()).map(|c| c.to_ascii_lowercase()).collect();
    cleaned.iter().eq(cleaned.iter().rev())
}

fn main() {
    let word = "héllo wörld";
    for (i, c) in word.char_indices().filter(|(_, c)| !c.is_ascii()) {
        println!("non-ascii {:?} at byte {}", c, i);
    }
    println!("chars: {}, bytes: {}", word.chars().count(), word.len());
    let reversed: String = word.chars().rev().collect();
    println!("reversed: {}", reversed);
    println!("upper: {}", word.to_uppercase());
    println!("lower: {}", "MiXeD CaSe".to_lowercase());
    println!("ascii upper: {}", "abc-é".to_ascii_uppercase());
    println!("caesar: {}", caesar("Hello, World!", 3));
    println!("decoded: {}", caesar(&caesar("Round Trip", 7), 19));
    for s in ["A man, a plan, a canal: Panama", "not one", "Was it a car or a cat I saw?"] {
        println!("{:?} palindrome? {}", s, is_palindrome(s));
    }
    let bytes = "AZaz09".bytes().collect::<Vec<u8>>();
    println!("bytes: {:?}", bytes);
    let sum: u32 = "hello".bytes().map(|b| b as u32).sum();
    println!("byte sum: {}", sum);
    println!("as_bytes[1]: {}", "hello".as_bytes()[1]);
    let line = "=".repeat(12);
    println!("{}", line);
    let alphabet: String = ('a'..='z').collect();
    println!("{}", alphabet);
    let step: String = ('a'..='z').step_by(5).collect();
    println!("{}", step);
    let mut counts = [0usize; 26];
    for c in "the quick brown fox".chars().filter(|c| c.is_ascii_alphabetic()) {
        counts[(c as u8 - b'a') as usize] += 1;
    }
    let top: Vec<String> = counts
        .iter()
        .enumerate()
        .filter(|(_, &n)| n > 1)
        .map(|(i, n)| format!("{}={}", (b'a' + i as u8) as char, n))
        .collect();
    println!("repeated: {}", top.join(" "));
    let title: String = "hello big world"
        .split(' ')
        .map(|w| {
            let mut cs = w.chars();
            match cs.next() {
                Some(f) => f.to_uppercase().collect::<String>() + cs.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    println!("title: {}", title);
    let mut s = String::from("abc");
    s.push('d');
    s.push_str("ef");
    s.insert(0, '>');
    s.insert_str(1, "> ");
    println!("{} len={} cap>={}", s, s.len(), s.capacity() >= s.len());
    let popped = s.pop();
    let removed = s.remove(0);
    println!("{:?} {:?} {}", popped, removed, s);
    s.truncate(3);
    println!("truncated: {:?}", s);
    s.retain(|c| c != ' ');
    println!("retained: {:?}", s);
    s.clear();
    println!("empty: {} {:?}", s.is_empty(), s);
    let digits: Vec<u32> = "4096".chars().filter_map(|c| c.to_digit(10)).collect();
    println!("digits: {:?}", digits);
    let hex: Vec<Option<u32>> = "fAg".chars().map(|c| c.to_digit(16)).collect();
    println!("hex: {:?}", hex);
    println!("from_digit: {:?} {:?}", std::char::from_digit(7, 10), char::from_digit(11, 16));
    println!("from_u32: {:?}", char::from_u32(0x41));
    println!("punct: {} {} {}", '!'.is_ascii_punctuation(), 'a'.is_ascii_punctuation(), ' '.is_whitespace());
    println!("numeric: {} {}", '7'.is_numeric(), 'x'.is_ascii_hexdigit());
    println!("char as u32: {}, u8 as char: {}", 'A' as u32, 97u8 as char);
    println!("len_utf8: {} {}", 'a'.len_utf8(), 'ö'.len_utf8());
    let mut chars: Vec<char> = "rustacean".chars().collect();
    chars.sort();
    chars.dedup();
    println!("sorted unique: {}", chars.iter().collect::<String>());
    println!("escape: {} {}", "a\nb".escape_debug(), "tab\t".escape_default());
    println!("char from u8 arithmetic: {}", (b'a'..=b'e').map(|b| (b - 32) as char).collect::<String>());
}
