fn main() {
    let a: usize = 3;
    let b: usize = 5;
    println!("{}", b - a);
    println!("{}", a.wrapping_sub(b));
    println!("{}", a.saturating_sub(b));
    println!("{:?}", a.checked_sub(b));
    let d = std::convert::identity(a) - b;
    println!("unreachable {}", d);
}
