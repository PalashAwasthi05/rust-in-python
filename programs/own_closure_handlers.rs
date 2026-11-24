use std::collections::HashMap;

struct Machine<'a> {
    handlers: HashMap<&'static str, Box<dyn FnMut(i64) -> i64 + 'a>>,
}

impl<'a> Machine<'a> {
    fn on(&mut self, name: &'static str, f: impl FnMut(i64) -> i64 + 'a) {
        self.handlers.insert(name, Box::new(f));
    }
    fn fire(&mut self, name: &str, x: i64) -> Option<i64> {
        self.handlers.get_mut(name).map(|h| h(x))
    }
}

fn counter_pair() -> (Box<dyn FnMut() -> u32>, Box<dyn Fn(u32) -> bool>) {
    let mut n = 0;
    let limit = 3;
    (Box::new(move || { n += 1; n }), Box::new(move |x| x >= limit))
}

fn make_accumulator(start: i32) -> impl FnMut(i32) -> i32 {
    let mut total = start;
    move |x| {
        total += x;
        total
    }
}

fn apply_all(fs: &mut [Box<dyn FnMut(i32) -> i32>], x: i32) -> Vec<i32> {
    fs.iter_mut().map(|f| f(x)).collect()
}

fn with_buffer<R>(f: impl FnOnce(&mut Vec<u8>) -> R) -> (R, Vec<u8>) {
    let mut buf = Vec::new();
    let r = f(&mut buf);
    (r, buf)
}

fn bump_all(counts: &mut [u32], mut by: impl FnMut(usize) -> u32) {
    for (i, c) in counts.iter_mut().enumerate() {
        *c += by(i);
    }
}

fn main() {
    let mut log: Vec<String> = Vec::new();
    {
        let mut m = Machine { handlers: HashMap::new() };
        let mut calls = 0;
        m.on("double", |x| x * 2);
        m.on("count", |x| {
            calls += 1;
            x + calls
        });
        let factor = 10;
        m.on("scale", move |x| x * factor);
        println!("{:?}", m.fire("double", 21));
        println!("{:?}", m.fire("count", 1));
        println!("{:?}", m.fire("count", 1));
        println!("{:?}", m.fire("scale", 4));
        println!("{:?}", m.fire("missing", 4));
        drop(m);
        log.push(format!("calls={}", calls));
    }
    println!("{:?}", log);

    let (mut next, done) = counter_pair();
    loop {
        let v = next();
        println!("tick {}", v);
        if done(v) {
            break;
        }
    }

    let mut acc = make_accumulator(100);
    acc(1);
    acc(2);
    println!("acc {}", acc(3));

    let mut fs: Vec<Box<dyn FnMut(i32) -> i32>> = Vec::new();
    for k in 1..=3 {
        let mut seen = 0;
        fs.push(Box::new(move |x| {
            seen += 1;
            x * k + seen
        }));
    }
    println!("{:?}", apply_all(&mut fs, 10));
    println!("{:?}", apply_all(&mut fs, 10));

    let (n, bytes) = with_buffer(|b| {
        b.extend_from_slice(b"hey");
        b.push(b'!');
        b.len()
    });
    println!("{} {:?}", n, bytes);

    let mut counts = vec![0u32; 4];
    let mut step = 0;
    bump_all(&mut counts, |i| {
        step += 1;
        (i as u32) * step
    });
    println!("{:?} step={}", counts, step);

    let words = vec!["a", "bb", "ccc"];
    let lens: HashMap<&str, usize> = words.iter().map(|w| (*w, w.len())).collect();
    let mut ks: Vec<_> = lens.iter().collect();
    ks.sort();
    println!("{:?}", ks);
    let lookup = |w: &str| lens.get(w).copied().unwrap_or(0);
    println!("{} {}", lookup("bb"), lookup("zz"));
}
