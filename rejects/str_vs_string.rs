// error: expected alloc::string::String, found &str
fn greet(name: String) { println!("hi {}", name); }
fn main() {
    println!("start");
    greet("bob");
}
