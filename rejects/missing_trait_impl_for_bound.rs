// error: Square does not implement Shape
trait Shape { fn area(&self) -> f64; }
struct Square(f64);
fn total<T: Shape>(s: &T) -> f64 { s.area() }
fn main() {
    println!("start");
    println!("{}", total(&Square(2.0)));
}
