// error: cannot assign to 'x' while 'x' is immutably borrowed
fn main() {
    println!("start");
    let mut x = 5;
    let y = &x;
    x += 1;
    println!("{} {}", x, y);
}
