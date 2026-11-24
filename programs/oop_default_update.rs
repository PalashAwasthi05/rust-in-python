// Default + struct update syntax `..Default::default()`, manual Default impls, nested defaults.
#[derive(Debug, Clone, PartialEq)]
enum Level {
    Error,
    Warn,
    Info,
    Debug,
}

impl Default for Level {
    fn default() -> Self {
        Level::Info
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct Retry {
    attempts: u8,
    backoff_ms: u64,
}

#[derive(Debug, Clone, Default)]
struct Config {
    name: String,
    port: u16,
    verbose: bool,
    level: Level,
    tags: Vec<String>,
    retry: Retry,
    ratio: f32,
    parent: Option<Box<Config>>,
}

#[derive(Debug)]
struct Window {
    title: &'static str,
    width: u32,
    height: u32,
    resizable: bool,
}

impl Default for Window {
    fn default() -> Self {
        Window { title: "untitled", width: 800, height: 600, resizable: true }
    }
}

#[derive(Debug, Default)]
struct Point3(i32, i32, i32);

#[derive(Debug, Default)]
enum Shape {
    #[default]
    Empty,
    Circle(f64),
}

fn make<T: Default + std::fmt::Debug>() -> T {
    T::default()
}

fn main() {
    let base = Config::default();
    println!("{:?}", base);

    let c = Config { name: "server".into(), port: 8080, ..Default::default() };
    println!("{} {} {:?} {}", c.name, c.port, c.level, c.verbose);

    let d = Config { verbose: true, level: Level::Debug, tags: vec!["x".into()], ..c.clone() };
    println!("{:#?}", d);

    let e = Config {
        retry: Retry { attempts: 3, ..Default::default() },
        parent: Some(Box::new(d.clone())),
        ..d
    };
    println!("retry={:?} parent={}", e.retry, e.parent.as_ref().map(|p| p.name.as_str()).unwrap_or("-"));
    println!("e.tags={:?} same retry? {}", e.tags, e.retry == Retry::default());

    let w = Window { title: "editor", ..Window::default() };
    let w2 = Window { width: 1024, height: 768, ..Default::default() };
    println!("{:?}\n{:?}", w, w2);

    let p: Point3 = Default::default();
    println!("{:?} {:?}", p, Shape::default());
    println!("{:?}", Shape::Circle(1.5));

    let n: i32 = make();
    let s: String = make();
    let v: Vec<u8> = make();
    let o: Option<Level> = make();
    let t: (bool, char, f64) = make();
    let l: Level = make();
    println!("{:?} {:?} {:?} {:?} {:?} {:?}", n, s, v, o, t, l);
    println!("{:?} {:?}", Level::Error, Level::Warn);

    let mut counts: Vec<u32> = Vec::new();
    counts.resize_with(3, Default::default);
    println!("{:?}", counts);
    let maybe: Option<Retry> = None;
    println!("{:?}", maybe.unwrap_or_default());
    let parsed: u16 = "abc".parse().unwrap_or_default();
    println!("{}", parsed);
}
