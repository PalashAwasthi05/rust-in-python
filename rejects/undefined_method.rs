// error: no method 'length' for Point
struct Point { x: i32 }
fn main() {
    println!("start");
    let p = Point { x: 1 };
    println!("{}", p.length());
}
