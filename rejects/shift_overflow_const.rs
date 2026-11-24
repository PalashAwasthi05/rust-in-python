// error: this arithmetic operation will overflow: attempt to shift left
fn main() {
    println!("start");
    let x: i32 = 1 << 40;
    println!("{}", x);
}
