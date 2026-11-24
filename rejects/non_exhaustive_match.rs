// error: non-exhaustive match, missing Color::Blue
enum Color { Red, Green, Blue }
fn name(c: Color) -> &'static str {
    match c {
        Color::Red => "red",
        Color::Green => "green",
    }
}
fn main() {
    println!("start");
    println!("{}", name(Color::Blue));
}
