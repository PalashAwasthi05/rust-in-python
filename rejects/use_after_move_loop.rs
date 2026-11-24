// error: use of moved value 's'
fn consume(s: String) { println!("{}", s); }
fn main() {
    println!("start");
    let s = String::from("x");
    for _ in 0..3 {
        consume(s);
    }
}
