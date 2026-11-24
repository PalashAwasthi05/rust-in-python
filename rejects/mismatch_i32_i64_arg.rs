// error: expected i64, found i32
fn take(x: i64) -> i64 { x * 2 }
fn main() {
    println!("start");
    let a: i32 = 5;
    println!("{}", take(a));
}
