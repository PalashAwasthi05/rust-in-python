// Closures capturing by shared ref, by unique ref and by move.
fn apply_n<F: FnMut()>(n: usize, mut f: F) {
    for _ in 0..n {
        f();
    }
}

fn call_twice<F>(f: F) -> i32
where
    F: Fn(i32) -> i32,
{
    f(f(1))
}

fn consume<F: FnOnce() -> String>(f: F) -> String {
    f()
}

#[derive(Debug)]
struct Config {
    factor: i32,
    label: String,
}

fn main() {
    let mut count = 0;
    let mut inc = || count += 1;
    inc();
    inc();
    apply_n(3, &mut inc);
    apply_n(2, inc);
    println!("count = {}", count);

    let cfg = Config { factor: 3, label: "scaled".to_string() };
    let scale = |x: i32| x * cfg.factor;
    println!("{} {}", scale(4), call_twice(scale));
    println!("still usable: {:?}", cfg);

    let mut log: Vec<String> = Vec::new();
    {
        let mut record = |msg: &str| log.push(format!("[{}] {}", log.len(), msg));
        record("one");
        record("two");
    }
    log.push("three".into());
    println!("{:?}", log);

    let name = String::from("world");
    let greet = move || format!("hello {}", name);
    println!("{}", greet());
    println!("{}", greet());
    println!("{}", consume(greet));

    let owned = vec![1, 2, 3];
    let total = consume(move || {
        let s: i32 = owned.iter().sum();
        format!("sum={}", s)
    });
    println!("{}", total);

    let threshold = 5;
    let mut values = vec![8, 2, 9, 4, 7, 1];
    values.retain(|&v| v > threshold || v % 2 == 0);
    values.sort_by_key(|&v| (v as i32 - threshold as i32).abs());
    println!("{:?}", values);

    let weights = vec![("x", 3), ("y", 1), ("z", 2)];
    let mut order = vec!["z", "x", "y"];
    order.sort_by_key(|name| weights.iter().find(|(n, _)| n == name).map(|(_, w)| *w).unwrap());
    println!("{:?}", order);
    println!("weights still here: {}", weights.len());

    let mut counter = 10;
    let mut dec = || {
        counter -= 1;
        counter
    };
    let a = dec();
    let b = dec();
    println!("{} {} {}", a, b, counter);

    let label = cfg.label;
    let describe = |n: i32| -> String { format!("{}:{}", label, n) };
    let outs: Vec<String> = (1..=3).map(describe).collect();
    println!("{:?} {}", outs, label);
}
