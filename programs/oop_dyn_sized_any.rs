// Trait objects with generic methods excluded via `where Self: Sized`, dyn Any downcasting,
// and the as_any pattern.
use std::any::Any;
use std::fmt::Debug;

trait Component: Debug {
    fn name(&self) -> String;
    fn update(&mut self, dt: f64);

    // Generic method: not object safe, so it is excluded from the vtable.
    fn with_name<F: Fn(&str) -> String>(&self, f: F) -> String
    where
        Self: Sized,
    {
        f(&self.name())
    }

    // Consuming method that needs a sized Self.
    fn boxed(self) -> Box<dyn Component>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[derive(Debug, Default)]
struct Position {
    x: f64,
    y: f64,
    vx: f64,
}

#[derive(Debug)]
struct Health(i32);

impl Component for Position {
    fn name(&self) -> String {
        "position".to_string()
    }
    fn update(&mut self, dt: f64) {
        self.x += self.vx * dt;
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Component for Health {
    fn name(&self) -> String {
        format!("health({})", self.0)
    }
    fn update(&mut self, _dt: f64) {
        if self.0 > 0 {
            self.0 -= 1;
        }
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

fn find<T: 'static>(components: &[Box<dyn Component>]) -> Option<&T> {
    components.iter().find_map(|c| c.as_any().downcast_ref::<T>())
}

fn describe(value: &dyn Any) -> String {
    if let Some(i) = value.downcast_ref::<i32>() {
        format!("i32 {}", i)
    } else if let Some(s) = value.downcast_ref::<String>() {
        format!("String {:?}", s)
    } else if let Some(s) = value.downcast_ref::<&str>() {
        format!("&str {:?}", s)
    } else if value.is::<f64>() {
        "some f64".to_string()
    } else {
        "unknown".to_string()
    }
}

fn main() {
    let p = Position { vx: 2.0, ..Default::default() };
    println!("{}", p.with_name(|n| n.to_uppercase()));
    let h = Health(3);
    println!("{}", h.with_name(|n| format!("<{}>", n)));

    let mut components: Vec<Box<dyn Component>> = vec![p.boxed(), h.boxed(), Box::new(Health(1))];
    for _ in 0..2 {
        for c in components.iter_mut() {
            c.update(0.5);
        }
    }
    for c in &components {
        println!("{} => {:?}", c.name(), c);
    }

    if let Some(pos) = find::<Position>(&components) {
        println!("found position at x={}", pos.x);
    }
    println!("first health: {:?}", find::<Health>(&components).map(|h| h.0));
    println!("is string? {}", find::<String>(&components).is_none());

    for c in components.iter_mut() {
        if let Some(h) = c.as_any_mut().downcast_mut::<Health>() {
            h.0 += 100;
        }
    }
    let healths: Vec<i32> =
        components.iter().filter_map(|c| c.as_any().downcast_ref::<Health>()).map(|h| h.0).collect();
    println!("healths: {:?}", healths);

    let values: Vec<Box<dyn Any>> =
        vec![Box::new(5i32), Box::new(String::from("hi")), Box::new("static"), Box::new(2.5f64), Box::new('c')];
    for v in &values {
        println!("{}", describe(v.as_ref()));
    }

    let boxed: Box<dyn Any> = Box::new(String::from("owned"));
    match boxed.downcast::<String>() {
        Ok(s) => println!("got back {} with len {}", s, s.len()),
        Err(_) => println!("not a string"),
    }
    let boxed: Box<dyn Any> = Box::new(7u8);
    match boxed.downcast::<String>() {
        Ok(s) => println!("got back {}", s),
        Err(other) => println!("not a string; u8? {}", other.is::<u8>()),
    }
}
