// error: expected an integer, found &str
fn main() {
    println!("start");
    let flag = true;
    let v = if flag { 1 } else { "one" };
    println!("{}", v);
}
