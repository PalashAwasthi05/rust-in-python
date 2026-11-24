// error: Point does not implement PartialEq
struct Point { x: i32, y: i32 }
fn main() {
    println!("start");
    let a = Point { x: 1, y: 2 };
    let b = Point { x: 1, y: 2 };
    println!("{}", a == b);
}
