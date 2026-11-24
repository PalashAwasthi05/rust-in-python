// Slice patterns over &[T], Vec via as_slice, and string slices.
fn describe(xs: &[i32]) -> String {
    match xs {
        [] => "empty".to_string(),
        [x] => format!("one: {}", x),
        [a, b] => format!("pair: {} {}", a, b),
        [first, .., last] if first == last => format!("bookends {}", first),
        [first, middle @ .., last] => format!("{}..{} ({} inside)", first, last, middle.len()),
    }
}

fn sum(xs: &[i32]) -> i32 {
    match xs {
        [] => 0,
        [head, tail @ ..] => *head + sum(tail),
    }
}

fn pairs(xs: &[i32]) -> Vec<(i32, i32)> {
    let mut out = Vec::new();
    let mut rest = xs;
    while let [a, b, tail @ ..] = rest {
        out.push((*a, *b));
        rest = tail;
    }
    if let [leftover] = rest {
        out.push((*leftover, 0));
    }
    out
}

fn command(words: &[&str]) -> String {
    match words {
        ["go", dir] => format!("going {}", dir),
        ["take", item, "from", place] => format!("taking {} from {}", item, place),
        ["say", rest @ ..] if !rest.is_empty() => format!("saying '{}'", rest.join(" ")),
        [verb, ..] => format!("unknown verb {}", verb),
        [] => "nothing".to_string(),
    }
}

fn is_palindrome(cs: &[char]) -> bool {
    match cs {
        [] | [_] => true,
        [a, inner @ .., b] => a == b && is_palindrome(inner),
    }
}

fn main() {
    let samples: Vec<Vec<i32>> = vec![vec![], vec![5], vec![1, 2], vec![7, 3, 7], vec![1, 2, 3, 4, 5]];
    for s in &samples {
        println!("{:?} -> {} (sum {})", s, describe(s), sum(s));
    }
    let v = vec![10, 20, 30];
    match v.as_slice() {
        [a, b, c] => println!("three: {}", a + b + c),
        _ => unreachable!(),
    }
    println!("{:?}", pairs(&[1, 2, 3, 4, 5]));
    println!("{:?}", pairs(&[9, 8]));
    for line in ["go north", "take lamp from table", "say hello there world", "dance wildly", ""] {
        let words: Vec<&str> = line.split_whitespace().collect();
        println!("{:>24} => {}", line, command(&words));
    }
    for w in ["racecar", "level", "rust", "a", ""] {
        let cs: Vec<char> = w.chars().collect();
        println!("{:?} palindrome? {}", w, is_palindrome(&cs));
    }
    let arr = [1u8, 2, 3, 4];
    let [x, y, ..] = arr;
    let [.., z] = arr;
    println!("{} {} {}", x, y, z);
    if let [first, rest @ ..] = &arr[..] {
        println!("first={} rest={:?}", first, rest);
    }
}
