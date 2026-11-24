// error: expected &str, found alloc::string::String
fn main() {
    println!("start");
    let a = String::from("a");
    let b = String::from("b");
    let c = a + b;
    println!("{}", c);
}
