// error: field 'radius' is private
mod shapes {
    pub struct Circle { radius: f64 }
    impl Circle {
        pub fn new(r: f64) -> Circle { Circle { radius: r } }
    }
}
fn main() {
    println!("start");
    let c = shapes::Circle::new(2.0);
    println!("{}", c.radius);
}
