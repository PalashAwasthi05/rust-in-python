// error: use of mutable static is unsafe
static mut N: i32 = 0;
fn main() {
    println!("start");
    N += 1;
}
