// error: unknown or ambiguous associated function 'Purple' for Color
enum Color { Red, Green }
fn main() {
    println!("start");
    let c = Color::Purple;
    match c { _ => println!("x") }
}
