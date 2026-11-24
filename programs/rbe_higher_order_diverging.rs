fn is_odd(n: u32) -> bool { n % 2 == 1 }

fn never_returns(msg: &str) -> ! {
    panic!("{}", msg)
}

fn parse_or_default(s: &str) -> u32 {
    match s.parse::<u32>() {
        Ok(v) => v,
        Err(_) => 0,
    }
}

fn sum_odd_numbers(up_to: u32) -> u32 {
    let mut acc = 0;
    for i in 0..up_to {
        // `continue` has type `!`, so it fits where a u32 is expected
        let addition: u32 = match i % 2 == 1 {
            true => i,
            false => continue,
        };
        acc += addition;
    }
    acc
}

fn first_word_len(s: &str) -> usize {
    let word = match s.split_whitespace().next() {
        Some(w) => w,
        None => return 0,
    };
    word.len()
}

fn checked_level(n: i32) -> &'static str {
    match n {
        0..=3 => "low",
        4..=6 => "mid",
        7..=9 => "high",
        _ => never_returns("level out of range"),
    }
}

fn main() {
    println!("Find the sum of all the numbers with odd squares under 1000");
    let upper = 1000;

    // imperative approach
    let mut acc = 0;
    for n in 0.. {
        let n_squared = n * n;
        if n_squared >= upper {
            break;
        } else if is_odd(n_squared) {
            acc += n_squared;
        }
    }
    println!("imperative style: {}", acc);

    // functional approach
    let sum_of_squared_odd_numbers: u32 = (0..)
        .map(|n| n * n)
        .take_while(|&n_squared| n_squared < upper)
        .filter(|&n_squared| is_odd(n_squared))
        .sum();
    println!("functional style: {}", sum_of_squared_odd_numbers);

    let words = ["apple", "Bob", "kayak", "Rust", "level", "noon"];
    let palindromes: Vec<String> = words
        .iter()
        .map(|w| w.to_lowercase())
        .filter(|w| w.chars().eq(w.chars().rev()))
        .collect();
    println!("palindromes: {:?}", palindromes);
    let longest = words.iter().copied().reduce(|a, b| if b.len() > a.len() { b } else { a });
    println!("longest: {:?}", longest);
    let grouped = words.iter().fold((0, 0), |(short, long), w| if w.len() <= 4 { (short + 1, long) } else { (short, long + 1) });
    println!("short/long counts: {:?}", grouped);
    let ops: [(&str, fn(i64, i64) -> i64); 3] = [("add", |a, b| a + b), ("mul", |a, b| a * b), ("max", i64::max)];
    for (name, op) in ops {
        println!("{} over [3, 7, 2] = {}", name, [3, 7, 2].into_iter().reduce(op).unwrap());
    }

    println!("sum of odd numbers up to 9 (excluding): {}", sum_odd_numbers(9));
    println!("parse_or_default: {} {}", parse_or_default("17"), parse_or_default("x"));
    println!("first word len: {} {}", first_word_len("  hello world"), first_word_len("   "));
    for n in [2, 5, 9] {
        println!("level {} is {}", n, checked_level(n));
    }
    let value: u32 = if upper > 0 { 7 } else { never_returns("unreachable") };
    println!("diverging branch typechecks as u32: {}", value);
    let x: Option<u32> = None;
    let y = x.unwrap_or_else(|| { let fallback = 99; fallback });
    println!("fallback {}", y);
}
