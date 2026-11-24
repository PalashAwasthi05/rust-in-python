// error: expected alloc::string::String, found &str
fn main() {
    println!("start");
    let s: String = "hello";
    println!("{}", s);
}
