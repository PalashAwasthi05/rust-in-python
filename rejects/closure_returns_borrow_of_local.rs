// error: closure may outlive the current function
fn make() -> impl Fn() -> usize {
    let s = String::new();
    || s.len()
}
fn main() {
    println!("start");
    println!("{}", make()());
}
