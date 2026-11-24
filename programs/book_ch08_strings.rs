// Chapter 8.2: storing UTF-8 encoded text with strings.
fn pig_latin(word: &str) -> String {
    let mut chars = word.chars();
    let first = match chars.next() {
        Some(c) => c,
        None => return String::new(),
    };
    if "aeiouAEIOU".contains(first) {
        format!("{word}-hay")
    } else {
        let rest: String = chars.collect();
        format!("{rest}-{}ay", first.to_ascii_lowercase())
    }
}

fn main() {
    let mut s = String::new();
    s.push_str("initial");
    println!("{s}");

    let data = "initial contents";
    let s = data.to_string();
    let s2 = "initial contents".to_string();
    let s3 = String::from("initial contents");
    println!("{} {} {}", s == s2, s2 == s3, s.len());

    let hellos = [
        "السلام عليكم", "Dobrý den", "Hello", "שלום", "नमस्ते", "こんにちは", "안녕하세요", "你好", "Olá",
        "Здравствуйте", "Hola",
    ];
    for h in hellos {
        println!("{h}: {} bytes, {} chars", h.len(), h.chars().count());
    }

    let mut s1 = String::from("foo");
    let s2 = "bar";
    s1.push_str(s2);
    println!("s1 is {s1}, s2 is {s2}");
    let mut s = String::from("lo");
    s.push('l');
    println!("{s}");

    let s1 = String::from("Hello, ");
    let s2 = String::from("world!");
    let s3 = s1 + &s2;
    println!("{s3} ({s2} still usable)");

    let s1 = String::from("tic");
    let s2 = String::from("tac");
    let s3 = String::from("toe");
    let s = s1 + "-" + &s2 + "-" + &s3;
    println!("{s}");

    let s1 = String::from("tic");
    let s = format!("{s1}-{s2}-{s3}");
    println!("{s} and s1 is still {s1}");

    let hello = "Здравствуйте";
    let s = &hello[0..4];
    println!("first 4 bytes: {s}");
    for c in "Зд".chars() {
        println!("{c}");
    }
    for b in "Зд".bytes() {
        println!("{b}");
    }
    let namaste = "नमस्ते";
    let chars: Vec<char> = namaste.chars().collect();
    println!("{:?}", chars);
    println!("{:?}", namaste.char_indices().map(|(i, _)| i).collect::<Vec<_>>());
    println!("is_char_boundary(1)? {}", hello.is_char_boundary(1));

    let words = ["first", "apple", "rust", "orange", "string"];
    let latin: Vec<String> = words.iter().map(|w| pig_latin(w)).collect();
    println!("{}", latin.join(" "));

    let sentence = "  The quick brown fox jumps over the lazy dog  ";
    let trimmed = sentence.trim();
    println!("[{trimmed}]");
    println!("{}", trimmed.to_uppercase());
    println!("{}", trimmed.replace("the", "a"));
    println!("{:?}", trimmed.split(' ').rev().collect::<Vec<_>>());
    println!("starts {} ends {} find {:?}", trimmed.starts_with("The"), trimmed.ends_with("dog"), trimmed.find("fox"));
    let caps: String = trimmed
        .split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    println!("{caps}");
    let mut owned = String::with_capacity(10);
    for (i, w) in trimmed.split_whitespace().enumerate() {
        if i > 0 {
            owned += "_";
        }
        owned += &w[..1];
    }
    println!("{owned}");
    let reversed: String = "stressed".chars().rev().collect();
    println!("{reversed}");
    let mut t = String::from("hello world");
    t.insert(0, '>');
    t.insert_str(1, "> ");
    let popped = t.pop();
    t.truncate(8);
    println!("{t:?} popped {popped:?}");
    t.clear();
    println!("empty? {}", t.is_empty());
}
