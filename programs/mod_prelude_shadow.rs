// User items that shadow prelude and std names.
use std::fmt::{self, Display};

#[derive(Debug)]
enum MyError {
    Negative(i32),
    TooBig,
}

impl Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MyError::Negative(n) => write!(f, "negative: {}", n),
            MyError::TooBig => write!(f, "too big"),
        }
    }
}

type Result<T> = std::result::Result<T, MyError>;

fn max(a: i32, b: i32) -> i32 {
    if a > b { a * 100 } else { b * 100 }
}

fn min<T: PartialOrd>(items: &[T]) -> Option<&T> {
    let mut best: Option<&T> = None;
    for it in items {
        best = match best {
            None => Some(it),
            Some(b) if it < b => Some(it),
            keep => keep,
        };
    }
    best
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Ordering {
    Before,
    Same,
    After,
}

fn compare(a: i32, b: i32) -> Ordering {
    match a.cmp(&b) {
        std::cmp::Ordering::Less => Ordering::Before,
        std::cmp::Ordering::Equal => Ordering::Same,
        std::cmp::Ordering::Greater => Ordering::After,
    }
}

#[derive(Debug)]
struct Entry {
    key: String,
    hits: u32,
}

struct Vec2 {
    x: i32,
    y: i32,
}

fn check(n: i32) -> Result<i32> {
    if n < 0 {
        Err(MyError::Negative(n))
    } else if n > 10 {
        Err(MyError::TooBig)
    } else {
        Ok(n * 2)
    }
}

fn run() -> Result<i32> {
    let a = check(3)?;
    let b = check(4)?;
    Ok(a + b)
}

fn main() {
    println!("{}", max(3, 9));
    println!("{}", std::cmp::max(3, 9));
    println!("{:?}", min(&[5, 2, 8]));
    println!("{:?} {:?} {:?}", compare(1, 2), compare(2, 2), compare(3, 2));
    println!("{}", compare(1, 1) == Ordering::Same);
    let e = Entry { key: "k".to_string(), hits: 3 };
    println!("{:?} {} {}", e, e.key, e.hits);
    let mut map = std::collections::HashMap::new();
    map.insert("x", 1);
    *map.entry("x").or_insert(0) += 10;
    *map.entry("y").or_insert(0) += 1;
    let mut keys: Vec<_> = map.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
    keys.sort();
    println!("{:?}", keys);
    let v = Vec2 { x: 1, y: 2 };
    let real: Vec<i32> = vec![v.x, v.y];
    println!("{:?}", real);
    for n in [3, -1, 20] {
        match check(n) {
            Ok(v) => println!("ok {}", v),
            Err(e) => println!("err {}", e),
        }
    }
    println!("{:?}", run());
    let std_res: std::result::Result<u8, String> = Err("plain".into());
    println!("{:?}", std_res);
}
