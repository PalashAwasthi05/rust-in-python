// error: cannot move 's' while 's' is immutably borrowed
fn main() {
    println!("start");
    let s = String::from("a");
    let r = &s;
    drop(s);
    println!("{}", r);
}
