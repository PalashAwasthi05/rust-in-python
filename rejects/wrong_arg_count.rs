// error: add takes 2 arguments
fn add(a: i32, b: i32) -> i32 { a + b }
fn main() {
    println!("start");
    println!("{}", add(1, 2, 3));
}
