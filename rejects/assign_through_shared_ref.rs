// error: cannot write through a shared reference
fn bump(n: &i32) {
    *n += 1;
}
fn main() {
    println!("start");
    let mut x = 1;
    bump(&x);
    println!("{}", x);
}
