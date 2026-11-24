use std::f64::consts::PI;

trait Shape {
    fn area(&self) -> f64;
    fn perimeter(&self) -> f64;
    fn name(&self) -> String;
    fn describe(&self) -> String {
        format!("{} with area {:.2} and perimeter {:.2}", self.name(), self.area(), self.perimeter())
    }
}

struct Circle { r: f64 }
struct Rect { w: f64, h: f64 }
struct Triangle { a: f64, b: f64, c: f64 }

impl Shape for Circle {
    fn area(&self) -> f64 { PI * self.r * self.r }
    fn perimeter(&self) -> f64 { 2.0 * PI * self.r }
    fn name(&self) -> String { "Circle".into() }
}

impl Shape for Rect {
    fn area(&self) -> f64 { self.w * self.h }
    fn perimeter(&self) -> f64 { 2.0 * (self.w + self.h) }
    fn name(&self) -> String {
        if self.w == self.h { "Square".to_string() } else { "Rectangle".to_string() }
    }
}

impl Shape for Triangle {
    fn area(&self) -> f64 {
        let s = self.perimeter() / 2.0;
        (s * (s - self.a) * (s - self.b) * (s - self.c)).sqrt()
    }
    fn perimeter(&self) -> f64 { self.a + self.b + self.c }
    fn name(&self) -> String { String::from("Triangle") }
    fn describe(&self) -> String {
        format!("Triangle({}, {}, {}) area {:.3}", self.a, self.b, self.c, self.area())
    }
}

fn largest<'a>(shapes: &'a [Box<dyn Shape>]) -> Option<&'a dyn Shape> {
    let mut best: Option<&dyn Shape> = None;
    for s in shapes {
        match best {
            Some(b) if b.area() >= s.area() => {}
            _ => best = Some(s.as_ref()),
        }
    }
    best
}

fn main() {
    let mut shapes: Vec<Box<dyn Shape>> = vec![
        Box::new(Circle { r: 1.5 }),
        Box::new(Rect { w: 3.0, h: 4.0 }),
        Box::new(Rect { w: 2.5, h: 2.5 }),
        Box::new(Triangle { a: 3.0, b: 4.0, c: 5.0 }),
    ];
    for s in &shapes {
        println!("{}", s.describe());
    }
    let total: f64 = shapes.iter().map(|s| s.area()).sum();
    println!("total area: {:.3}", total);
    if let Some(big) = largest(&shapes) {
        println!("largest: {}", big.name());
    }
    shapes.sort_by(|a, b| a.perimeter().partial_cmp(&b.perimeter()).unwrap());
    let names: Vec<String> = shapes.iter().map(|s| s.name()).collect();
    println!("by perimeter: {}", names.join(", "));
}
