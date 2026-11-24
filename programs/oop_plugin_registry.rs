// Plugin registry: HashMap<String, Box<dyn Fn() -> Box<dyn Shape>>> factories and a
// BTreeMap of named handlers built with closures capturing configuration.
use std::collections::{BTreeMap, HashMap};

trait Shape {
    fn area(&self) -> f64;
    fn describe(&self) -> String;
}

struct Square(f64);
struct Circle(f64);
struct Triangle {
    base: f64,
    height: f64,
}

impl Shape for Square {
    fn area(&self) -> f64 {
        self.0 * self.0
    }
    fn describe(&self) -> String {
        format!("square side {}", self.0)
    }
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.0 * self.0
    }
    fn describe(&self) -> String {
        format!("circle radius {}", self.0)
    }
}

impl Shape for Triangle {
    fn area(&self) -> f64 {
        0.5 * self.base * self.height
    }
    fn describe(&self) -> String {
        format!("triangle {}x{}", self.base, self.height)
    }
}

type Factory = Box<dyn Fn() -> Box<dyn Shape>>;

struct Registry {
    factories: HashMap<String, Factory>,
}

impl Registry {
    fn new() -> Self {
        Registry { factories: HashMap::new() }
    }

    fn register<F>(&mut self, name: &str, f: F)
    where
        F: Fn() -> Box<dyn Shape> + 'static,
    {
        if self.factories.insert(name.to_string(), Box::new(f)).is_some() {
            println!("replaced plugin {}", name);
        }
    }

    fn create(&self, name: &str) -> Result<Box<dyn Shape>, String> {
        self.factories.get(name).map(|f| f()).ok_or_else(|| format!("no plugin named {:?}", name))
    }

    fn names(&self) -> Vec<&String> {
        let mut names: Vec<&String> = self.factories.keys().collect();
        names.sort();
        names
    }
}

type Handler = Box<dyn Fn(&[&str]) -> Result<String, String>>;

fn build_commands(prefix: String) -> BTreeMap<&'static str, Handler> {
    let mut cmds: BTreeMap<&'static str, Handler> = BTreeMap::new();
    cmds.insert("echo", Box::new(move |args| Ok(format!("{}{}", prefix, args.join(" ")))));
    cmds.insert(
        "sum",
        Box::new(|args| {
            let mut total = 0i64;
            for a in args {
                total += a.parse::<i64>().map_err(|e| format!("bad number {:?}: {}", a, e))?;
            }
            Ok(total.to_string())
        }),
    );
    cmds.insert("count", Box::new(|args| Ok(args.len().to_string())));
    cmds
}

fn main() {
    let mut reg = Registry::new();
    reg.register("square", || Box::new(Square(2.0)));
    reg.register("circle", || Box::new(Circle(1.0)));
    let default_base = 3.0;
    reg.register("triangle", move || Box::new(Triangle { base: default_base, height: 4.0 }));
    reg.register("unit", || Box::new(Square(1.0)));
    reg.register("square", || Box::new(Square(5.0)));
    println!("plugins: {:?}", reg.names());

    for name in ["square", "circle", "triangle", "hexagon", "unit"] {
        match reg.create(name) {
            Ok(shape) => println!("{:>8}: {} area={:.3}", name, shape.describe(), shape.area()),
            Err(e) => println!("{:>8}: error: {}", name, e),
        }
    }

    let shapes: Vec<Box<dyn Shape>> = reg.names().iter().filter_map(|n| reg.create(n).ok()).collect();
    let total: f64 = shapes.iter().map(|s| s.area()).sum();
    println!("total area of one of each: {:.2}", total);

    let cmds = build_commands(String::from("> "));
    let script = ["echo hello world", "sum 1 2 39", "sum 4 x", "count a b c d", "frobnicate now"];
    for line in script.iter() {
        let mut parts = line.split_whitespace();
        let cmd = parts.next().unwrap();
        let args: Vec<&str> = parts.collect();
        let out = match cmds.get(cmd) {
            Some(h) => h(&args),
            None => Err(format!("unknown command {}", cmd)),
        };
        println!("{:<18} => {:?}", line, out);
    }
    println!("commands: {:?}", cmds.keys().collect::<Vec<_>>());
}
