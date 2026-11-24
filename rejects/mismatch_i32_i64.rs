// error: expected i32, found i64
fn main() {
    println!("start");
    let a: i32 = 5;
    let b: i64 = 10;
    let c = a + b;
    println!("{}", c);
}
