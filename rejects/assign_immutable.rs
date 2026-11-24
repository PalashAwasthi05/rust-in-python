// error: cannot assign to immutable binding 'x'
fn main() {
    println!("start");
    let x = 5;
    x = 6;
    println!("{}", x);
}
