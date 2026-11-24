use std::f64::consts::PI;
use std::fmt;

trait Shape: fmt::Debug {
    fn area(&self) -> f64;
    fn perimeter(&self) -> f64;
    fn name(&self) -> String {
        String::from("shape")
    }
    fn describe(&self) -> String {
        format!("{} with area {:.2} and perimeter {:.2}", self.name(), self.area(), self.perimeter())
    }
}

#[derive(Debug)]
struct Circle {
    r: f64,
}

#[derive(Debug)]
struct Rect {
    w: f64,
    h: f64,
}

#[derive(Debug)]
struct Triangle {
    a: f64,
    b: f64,
    c: f64,
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        PI * self.r * self.r
    }
    fn perimeter(&self) -> f64 {
        2.0 * PI * self.r
    }
    fn name(&self) -> String {
        "circle".to_string()
    }
}

impl Shape for Rect {
    fn area(&self) -> f64 {
        self.w * self.h
    }
    fn perimeter(&self) -> f64 {
        2.0 * (self.w + self.h)
    }
    fn name(&self) -> String {
        if self.w == self.h { "square".into() } else { "rectangle".into() }
    }
}

impl Shape for Triangle {
    fn area(&self) -> f64 {
        let s = self.perimeter() / 2.0;
        (s * (s - self.a) * (s - self.b) * (s - self.c)).sqrt()
    }
    fn perimeter(&self) -> f64 {
        self.a + self.b + self.c
    }
}

fn largest<'a>(shapes: &'a [Box<dyn Shape>]) -> &'a dyn Shape {
    let mut best = &shapes[0];
    for s in shapes.iter() {
        if s.area() > best.area() {
            best = s;
        }
    }
    best.as_ref()
}

fn total_area(shapes: &[&dyn Shape]) -> f64 {
    shapes.iter().map(|s| s.area()).sum()
}

fn main() {
    let shapes: Vec<Box<dyn Shape>> = vec![
        Box::new(Circle { r: 1.5 }),
        Box::new(Rect { w: 3.0, h: 4.0 }),
        Box::new(Rect { w: 2.0, h: 2.0 }),
        Box::new(Triangle { a: 3.0, b: 4.0, c: 5.0 }),
    ];
    for s in &shapes {
        println!("{}", s.describe());
    }
    let big = largest(&shapes);
    println!("largest: {:?}", big);
    let c = Circle { r: 2.0 };
    let r = Rect { w: 1.0, h: 5.0 };
    let refs: Vec<&dyn Shape> = vec![&c, &r];
    println!("total = {:.3}", total_area(&refs));
    let mut sorted: Vec<&Box<dyn Shape>> = shapes.iter().collect();
    sorted.sort_by(|a, b| a.area().partial_cmp(&b.area()).unwrap());
    let names: Vec<String> = sorted.iter().map(|s| s.name()).collect();
    println!("{}", names.join(" < "));
    println!("{:?}", shapes);
    println!("{:?}", refs);
}
