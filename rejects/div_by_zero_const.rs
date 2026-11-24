// error: will panic at runtime: attempt to divide by zero
fn main() {
    println!("start");
    let x = 1 / 0;
    println!("{}", x);
}
