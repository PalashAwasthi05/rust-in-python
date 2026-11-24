// error: use of moved value 's'
fn main() {
    println!("start");
    let s = String::from("hello");
    let t = s;
    println!("{} {}", s, t);
}
