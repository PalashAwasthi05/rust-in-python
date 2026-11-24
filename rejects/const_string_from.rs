// error: cannot call non-const function 'String::from' in consts
const S: String = String::from("x");
fn main() {
    println!("start");
    println!("{}", S);
}
