// Closures stored in struct fields and collections of Box<dyn FnMut>.
use std::collections::HashMap;

struct Button {
    label: String,
    on_click: Box<dyn Fn(&str) -> String>,
}

struct EventBus {
    handlers: Vec<Box<dyn FnMut(i32)>>,
}

impl EventBus {
    fn new() -> Self {
        EventBus { handlers: Vec::new() }
    }
    fn subscribe(&mut self, f: impl FnMut(i32) + 'static) {
        self.handlers.push(Box::new(f));
    }
    fn emit(&mut self, v: i32) {
        for h in self.handlers.iter_mut() {
            h(v);
        }
    }
}

struct Cache<F: Fn(u64) -> u64> {
    func: F,
    memo: HashMap<u64, u64>,
    calls: u32,
}

impl<F: Fn(u64) -> u64> Cache<F> {
    fn new(func: F) -> Self {
        Cache { func, memo: HashMap::new(), calls: 0 }
    }
    fn get(&mut self, n: u64) -> u64 {
        if let Some(&v) = self.memo.get(&n) {
            return v;
        }
        self.calls += 1;
        let v = (self.func)(n);
        self.memo.insert(n, v);
        v
    }
}

struct Validator<'a> {
    rules: Vec<(&'a str, Box<dyn Fn(&str) -> bool + 'a>)>,
}

impl<'a> Validator<'a> {
    fn check(&self, input: &str) -> Vec<&'a str> {
        self.rules.iter().filter(|(_, r)| !r(input)).map(|(n, _)| *n).collect()
    }
}

fn main() {
    let b = Button {
        label: "ok".to_string(),
        on_click: Box::new(|who| format!("clicked by {}", who)),
    };
    println!("{}: {}", b.label, (b.on_click)("ann"));

    let mut bus = EventBus::new();
    let mut total = 0;
    bus.subscribe(|v| println!("got {}", v));
    let mut seen = Vec::new();
    bus.subscribe(move |v| {
        seen.push(v);
        println!("seen so far {:?}", seen);
    });
    bus.emit(1);
    bus.emit(2);
    let mut adders: Vec<Box<dyn FnMut(i32) -> i32>> = Vec::new();
    for step in 1..=3 {
        let mut acc = 0;
        adders.push(Box::new(move |x| {
            acc += x * step;
            acc
        }));
    }
    for round in 0..2 {
        for a in adders.iter_mut() {
            total += a(round + 1);
        }
    }
    println!("total = {}", total);

    let mut cache = Cache::new(|n| (1..=n).product());
    println!("{} {} {}", cache.get(5), cache.get(10), cache.get(5));
    println!("calls = {}", cache.calls);

    let min_len = 3;
    let banned = vec!["root", "admin"];
    let v = Validator {
        rules: vec![
            ("too short", Box::new(|s: &str| s.len() >= min_len)),
            ("banned", Box::new(|s: &str| !banned.contains(&s))),
            ("lowercase", Box::new(|s: &str| s.chars().all(|c| c.is_lowercase()))),
        ],
    };
    for name in ["ab", "admin", "Zed", "carol"] {
        println!("{:>6}: {:?}", name, v.check(name));
    }

    let mut ops: HashMap<&str, fn(i32, i32) -> i32> = HashMap::new();
    ops.insert("add", |a, b| a + b);
    ops.insert("sub", |a, b| a - b);
    let mut names: Vec<_> = ops.keys().collect();
    names.sort();
    for n in names {
        println!("{} {}", n, ops[n](10, 4));
    }
}
