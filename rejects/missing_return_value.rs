// error: expected i32, found \(\)
fn double(x: i32) -> i32 {
    x * 2;
}
fn main() {
    println!("start");
    println!("{}", double(4));
}
