// Clone on Box<dyn Trait> via a clone_box helper trait, and comparing trait objects.
use std::fmt;

trait Shape: ShapeClone + fmt::Debug {
    fn area(&self) -> f64;
    fn name(&self) -> &str;
    fn scale(&mut self, k: f64);
    fn key(&self) -> (String, u64) {
        (self.name().to_string(), (self.area() * 1000.0).round() as u64)
    }
}

trait ShapeClone {
    fn clone_box(&self) -> Box<dyn Shape>;
}

impl<T> ShapeClone for T
where
    T: 'static + Shape + Clone,
{
    fn clone_box(&self) -> Box<dyn Shape> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Shape> {
    fn clone(&self) -> Box<dyn Shape> {
        self.clone_box()
    }
}

impl PartialEq for dyn Shape {
    fn eq(&self, other: &dyn Shape) -> bool {
        self.key() == other.key()
    }
}

impl fmt::Display for dyn Shape {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} with area {:.2}", self.name(), self.area())
    }
}

#[derive(Debug, Clone)]
struct Circle {
    r: f64,
}

#[derive(Debug, Clone)]
struct Rect {
    w: f64,
    h: f64,
    label: String,
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        3.14159 * self.r * self.r
    }
    fn name(&self) -> &str {
        "circle"
    }
    fn scale(&mut self, k: f64) {
        self.r *= k;
    }
}

impl Shape for Rect {
    fn area(&self) -> f64 {
        self.w * self.h
    }
    fn name(&self) -> &str {
        &self.label
    }
    fn scale(&mut self, k: f64) {
        self.w *= k;
        self.h *= k;
    }
}

#[derive(Clone, Debug)]
struct Scene {
    shapes: Vec<Box<dyn Shape>>,
}

fn largest(shapes: &[Box<dyn Shape>]) -> Option<&Box<dyn Shape>> {
    shapes
        .iter()
        .max_by(|a, b| a.area().partial_cmp(&b.area()).unwrap())
}

fn main() {
    let original = Scene {
        shapes: vec![
            Box::new(Circle { r: 1.0 }),
            Box::new(Rect { w: 2.0, h: 3.0, label: "rect".into() }),
            Box::new(Rect { w: 1.0, h: 1.0, label: "square".into() }),
        ],
    };
    let mut copy = original.clone();
    for s in copy.shapes.iter_mut() {
        s.scale(2.0);
    }
    copy.shapes.push(original.shapes[0].clone());

    for s in &original.shapes {
        println!("original: {}", s);
    }
    for s in &copy.shapes {
        println!("copy: {}", s);
    }
    println!("{:?}", original);

    let a: Box<dyn Shape> = Box::new(Circle { r: 1.0 });
    let b = a.clone();
    let c: Box<dyn Shape> = Box::new(Rect { w: 3.14159, h: 1.0, label: "circle".into() });
    println!("a == b: {}", *a == *b);
    println!("a == c: {}", &*a == &*c);
    println!("a == copy[0]: {}", *a == *copy.shapes[0]);
    println!("a == copy[3]: {}", *a == *copy.shapes[3]);

    if let Some(big) = largest(&copy.shapes) {
        println!("largest: {}", big);
    }
    let total: f64 = copy.shapes.iter().map(|s| s.area()).sum();
    println!("total area {:.3}", total);
    let names: Vec<&str> = original.shapes.iter().map(|s| s.name()).collect();
    println!("{}", names.join(","));
    let found = original.shapes.iter().position(|s| **s == *b);
    println!("position of b: {:?}", found);
}
