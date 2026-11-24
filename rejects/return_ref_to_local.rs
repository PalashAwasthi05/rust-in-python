// error: reference to 's' outlives its block
fn make() -> &'static String {
    let s = String::from("hi");
    &s
}
fn main() {
    println!("start");
    println!("{}", make());
}
