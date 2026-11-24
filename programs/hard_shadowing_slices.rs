fn summarize(xs: &[i32]) -> String {
    match xs {
        [] => "nothing".to_string(),
        [only] => format!("just {}", only),
        [first, second] => format!("{} then {}", first, second),
        [first, .., last] if first == last => format!("{} items bookended by {}", xs.len(), first),
        [first, middle @ .., last] => format!("{}..{} with middle sum {}", first, last, middle.iter().sum::<i32>()),
    }
}

fn sum_recursive(xs: &[i32]) -> i32 {
    match xs {
        [] => 0,
        [x, rest @ ..] => x + sum_recursive(rest),
    }
}

fn is_sorted(xs: &[i32]) -> bool {
    match xs {
        [] | [_] => true,
        [a, b, rest @ ..] => a <= b && is_sorted(&xs[1..]) && rest.len() < xs.len(),
    }
}

fn parse_cmd(parts: &[&str]) -> String {
    match parts {
        ["get", key] => format!("GET {}", key),
        ["set", key, value] => format!("SET {}={}", key, value),
        ["del", keys @ ..] if !keys.is_empty() => format!("DEL {}", keys.join(",")),
        [cmd, ..] => format!("unknown {:?} ({} args)", cmd, parts.len() - 1),
        [] => "empty".into(),
    }
}

fn main() {
    for xs in [&[][..], &[4], &[1, 2], &[7, 3, 7], &[1, 2, 3, 4, 5]] {
        println!("{:?}: {}", xs, summarize(xs));
    }
    println!("recursive sum {}", sum_recursive(&[3, 4, 5, 6]));
    println!("sorted? {} {}", is_sorted(&[1, 2, 2, 9]), is_sorted(&[3, 1]));
    for line in ["get name", "set name ferris", "del a b c", "del", "ping", ""] {
        let parts: Vec<&str> = line.split_whitespace().collect();
        println!("{:<16} -> {}", line, parse_cmd(&parts));
    }
    if let [a, b, ..] = "x,y,z".split(',').collect::<Vec<_>>()[..] {
        println!("first two: {} {}", a, b);
    }
    let arr = [10, 20, 30, 40];
    let [head, tail @ ..] = arr;
    println!("head {} tail {:?}", head, tail);

    let value = "42";
    let value: i32 = value.parse().unwrap();
    let value = value * 2;
    let value = {
        let doubled = value * 2;
        let value = doubled + 1;
        value - 3
    };
    println!("shadowed value: {}", value);
    let label = if value > 100 { "big" } else { "small" };
    let tier = match value { v if v % 2 == 0 => v / 2, v => v };
    let config = {
        let base = 8;
        let factor = if label == "big" { 4 } else { 2 };
        (base * factor, label.to_uppercase())
    };
    println!("label {} tier {} config {:?}", label, tier, config);
    let x = 5;
    let y = { let x = x * 3; x + 1 };
    println!("outer x {} y {}", x, y);
    let total = { let mut t = 0; for i in 1..=4 { t += i * i; } t };
    let s = String::from("hello");
    let s = s.len();
    println!("total {} s {}", total, s);
}
