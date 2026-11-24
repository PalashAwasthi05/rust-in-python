// error: will panic at runtime: index out of bounds
fn main() {
    println!("start");
    let a = [1, 2, 3];
    println!("{}", a[5]);
}
