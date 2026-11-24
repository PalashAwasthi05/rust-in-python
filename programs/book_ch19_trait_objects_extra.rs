// Chapter 17/19 extras: user traits with default type parameters, impls on dyn Trait,
// trait objects with generic methods kept off the vtable, and dyn Any.
use std::any::Any;
use std::collections::HashMap;
use std::fmt;

trait Combine<Rhs = Self> {
    type Output;
    fn combine(&self, rhs: &Rhs) -> Self::Output;
}

#[derive(Debug, Clone, PartialEq)]
struct Tags(Vec<String>);

impl Combine for Tags {
    type Output = Tags;
    fn combine(&self, rhs: &Tags) -> Tags {
        let mut all = self.0.clone();
        for t in &rhs.0 {
            if !all.contains(t) {
                all.push(t.clone());
            }
        }
        Tags(all)
    }
}

impl Combine<&str> for Tags {
    type Output = usize;
    fn combine(&self, rhs: &&str) -> usize {
        self.0.iter().filter(|t| t.as_str() == *rhs).count()
    }
}

trait Shape {
    fn area(&self) -> f64;
    fn name(&self) -> String;
    fn scaled(&self, k: f64) -> Box<dyn Shape>;
}

impl fmt::Debug for dyn Shape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{} area={:.2}>", self.name(), self.area())
    }
}

impl dyn Shape {
    fn bigger_than(&self, other: &dyn Shape) -> bool {
        self.area() > other.area()
    }
}

struct Circle {
    r: f64,
}

struct Rect {
    w: f64,
    h: f64,
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        3.14159 * self.r * self.r
    }
    fn name(&self) -> String {
        String::from("circle")
    }
    fn scaled(&self, k: f64) -> Box<dyn Shape> {
        Box::new(Circle { r: self.r * k })
    }
}

impl Shape for Rect {
    fn area(&self) -> f64 {
        self.w * self.h
    }
    fn name(&self) -> String {
        format!("rect {}x{}", self.w, self.h)
    }
    fn scaled(&self, k: f64) -> Box<dyn Shape> {
        Box::new(Rect { w: self.w * k, h: self.h * k })
    }
}

trait Plugin {
    fn id(&self) -> &str;
    fn run(&mut self, input: i64) -> i64;
}

struct Doubler {
    calls: u32,
}

struct Offset(i64);

impl Plugin for Doubler {
    fn id(&self) -> &str {
        "doubler"
    }
    fn run(&mut self, input: i64) -> i64 {
        self.calls += 1;
        input * 2
    }
}

impl Plugin for Offset {
    fn id(&self) -> &str {
        "offset"
    }
    fn run(&mut self, input: i64) -> i64 {
        input + self.0
    }
}

fn describe_any(value: &dyn Any) -> String {
    if let Some(n) = value.downcast_ref::<i32>() {
        format!("i32 {n}")
    } else if let Some(s) = value.downcast_ref::<String>() {
        format!("String {s:?}")
    } else {
        String::from("something else")
    }
}

fn main() {
    let a = Tags(vec!["rust".into(), "book".into()]);
    let b = Tags(vec!["book".into(), "async".into()]);
    println!("{:?}", a.combine(&b));
    println!("{}", a.combine(&"rust"));
    println!("{}", Combine::<&str>::combine(&b, &"missing"));

    let shapes: Vec<Box<dyn Shape>> = vec![Box::new(Circle { r: 1.0 }), Box::new(Rect { w: 2.0, h: 3.0 })];
    for s in &shapes {
        println!("{:?}", s);
    }
    let grown: Vec<Box<dyn Shape>> = shapes.iter().map(|s| s.scaled(2.0)).collect();
    for g in grown.iter() {
        println!("grown {:?}", g);
    }
    println!("rect bigger than circle? {}", shapes[1].bigger_than(shapes[0].as_ref()));
    let largest = grown
        .iter()
        .max_by(|x, y| x.area().partial_cmp(&y.area()).unwrap())
        .unwrap();
    println!("largest {}", largest.name());

    let mut plugins: Vec<Box<dyn Plugin>> = vec![Box::new(Doubler { calls: 0 }), Box::new(Offset(-3))];
    let mut value = 5;
    for _ in 0..3 {
        for p in plugins.iter_mut() {
            value = p.run(value);
        }
    }
    println!("value {value}");
    let mut by_id: HashMap<String, usize> = HashMap::new();
    for (i, p) in plugins.iter().enumerate() {
        by_id.insert(p.id().to_string(), i);
    }
    let mut ids: Vec<_> = by_id.into_iter().collect();
    ids.sort();
    println!("{:?}", ids);

    let values: Vec<Box<dyn Any>> = vec![Box::new(7i32), Box::new(String::from("seven")), Box::new(7.0f64)];
    for v in &values {
        println!("{}", describe_any(v.as_ref()));
    }
    println!("{}", values[0].is::<i32>());
    println!("{:?}", values[2]);
}
