// error: Point has no field 'z'
struct Point { x: i32, y: i32 }
fn main() {
    println!("start");
    let p = Point { x: 1, y: 2 };
    println!("{}", p.z);
}
