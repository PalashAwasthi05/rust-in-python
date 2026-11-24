// error: closure may outlive the current function
// known-gap: E0373: non-move closure returned from a fn borrows a local
fn make() -> impl Fn() -> usize {
    let s = String::new();
    || s.len()
}
fn main() {
    println!("start");
    println!("{}", make()());
}
