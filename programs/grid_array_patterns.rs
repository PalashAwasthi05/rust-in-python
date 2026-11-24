// Array/slice pattern matching and destructuring.
#[derive(Debug)]
struct Rgb([u8; 3]);

fn classify(a: [i32; 3]) -> &'static str {
    match a {
        [0, 0, 0] => "zero",
        [x, y, z] if x == y && y == z => "uniform",
        [_, 0, _] => "middle zero",
        [a, .., c] if a > c => "descending ends",
        _ => "other",
    }
}

fn head_tail(s: &[u8]) -> String {
    match s {
        [] => String::from("nothing"),
        [only] => format!("only {}", only),
        [first, rest @ ..] => format!("{} then {:?}", first, rest),
    }
}

fn mid(s: &[i32]) -> Option<&[i32]> {
    if let [_, middle @ .., _] = s {
        Some(middle)
    } else {
        None
    }
}

fn main() {
    let arr = [1, 2, 3];
    let [a, b, c] = arr;
    println!("{} {} {}", a, b, c);
    let [first, .., last] = [10, 20, 30, 40, 50];
    println!("{} {}", first, last);
    let [x, rest @ ..] = [7u8, 8, 9];
    println!("{} {:?}", x, rest);
    let [.., y, z] = ["p", "q", "r", "s"];
    println!("{} {}", y, z);

    for t in [[0, 0, 0], [4, 4, 4], [1, 0, 9], [9, 5, 1], [1, 2, 3]] {
        println!("{:?} -> {}", t, classify(t));
    }
    println!("{}", head_tail(&[]));
    println!("{}", head_tail(&[5]));
    println!("{}", head_tail(&[5, 6, 7]));
    println!("{:?} {:?} {:?}", mid(&[1, 2, 3, 4]), mid(&[1, 2]), mid(&[1]));

    let color = Rgb([255, 128, 0]);
    let Rgb([r, g, bl]) = color;
    println!("#{:02x}{:02x}{:02x}", r, g, bl);
    println!("{:?}", color);

    let pairs = [[1, 2], [3, 4], [5, 6]];
    let mut s = 0;
    for [p, q] in pairs {
        s += p * q;
    }
    println!("dot {}", s);
    for &[p, q] in pairs.iter() {
        print!("({},{}) ", q, p);
    }
    println!();

    let grid = [[1, 2], [3, 4]];
    let [[a1, a2], [b1, b2]] = grid;
    println!("det {}", a1 * b2 - a2 * b1);
    if let [[1, _], row] = grid {
        println!("starts with 1, second row {:?}", row);
    }

    let mut buf = [0u8; 4];
    let [ref mut h, .., ref mut t] = buf;
    *h = 1;
    *t = 9;
    println!("{:?}", buf);
    if let [head, ..] = &mut buf {
        *head += 10;
    }
    println!("{:?}", buf);

    let v = vec![3, 1, 4, 1, 5, 9];
    match v.as_slice() {
        [a, b, rest @ ..] if a > b => println!("{} > {} and {} more", a, b, rest.len()),
        _ => println!("no"),
    }
    let (left, right): ([i32; 2], [i32; 2]) = ([1, 2], [3, 4]);
    println!("{:?} {:?}", left, right);
}
