fn main() {
    let s = String::from("añb");
    println!("first: {}", &s[0..1]);
    let n = s.len();
    println!("len {}", n);
    let bad = &s[0..2];
    println!("unreachable {}", bad);
}
