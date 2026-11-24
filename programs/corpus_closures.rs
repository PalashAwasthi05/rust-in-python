struct Button {
    label: String,
    on_click: Box<dyn Fn(u32) -> String>,
}

struct Counter {
    count: u32,
    step: Box<dyn FnMut() -> u32>,
}

impl Counter {
    fn tick(&mut self) -> u32 {
        self.count += (self.step)();
        self.count
    }
}

fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}

fn make_multiplier(n: i32) -> Box<dyn Fn(i32) -> i32> {
    Box::new(move |x| x * n)
}

fn compose<A, B, C>(f: impl Fn(A) -> B, g: impl Fn(B) -> C) -> impl Fn(A) -> C {
    move |x| g(f(x))
}

fn apply_twice<F>(f: F, x: i32) -> i32 where F: Fn(i32) -> i32 {
    f(f(x))
}

fn main() {
    let prefix = String::from(">>");
    let buttons = vec![
        Button { label: "save".into(), on_click: Box::new(|n| format!("saved {} times", n)) },
        Button { label: "greet".into(), on_click: Box::new(move |n| format!("{} hello #{}", prefix, n)) },
    ];
    for (i, b) in buttons.iter().enumerate() {
        println!("[{}] {}", b.label, (b.on_click)(i as u32 + 1));
    }

    let mut growth = 0;
    let mut c = Counter { count: 0, step: Box::new(move || { growth += 1; growth }) };
    let ticks: Vec<u32> = (0..5).map(|_| c.tick()).collect();
    println!("ticks: {:?}", ticks);

    let add5 = make_adder(5);
    let triple = make_multiplier(3);
    println!("add5(10)={} triple(7)={}", add5(10), triple(7));
    let both = compose(add5, triple);
    println!("compose(2)={}", both(2));
    println!("apply_twice(triple, 2)={}", apply_twice(&*make_multiplier(3), 2));

    let ops: Vec<(&str, Box<dyn Fn(i32, i32) -> i32>)> = vec![
        ("add", Box::new(|a, b| a + b)),
        ("sub", Box::new(|a, b| a - b)),
        ("max", Box::new(|a: i32, b| a.max(b))),
        ("pow", Box::new(|a: i32, b| a.pow(b as u32))),
    ];
    for (name, op) in &ops {
        println!("{}(6, 3) = {}", name, op(6, 3));
    }

    let mut log: Vec<String> = Vec::new();
    let mut record = |msg: &str| log.push(msg.to_uppercase());
    record("start");
    record("stop");
    println!("log: {:?}", log);

    let threshold = 10;
    let big: Vec<i32> = vec![4, 15, 8, 23, 10].into_iter().filter(|&x| x > threshold).collect();
    println!("big: {:?}", big);
    let consume = move || big.len();
    println!("consumed len {}", consume());
}
