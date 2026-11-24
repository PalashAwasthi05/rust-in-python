// error: reference does not live for 'static
fn main() {
    println!("start");
    let s: &'static str = &String::from("a");
    println!("{}", s);
}
