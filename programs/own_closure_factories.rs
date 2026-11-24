// Returning closures from functions: impl Fn, Box<dyn Fn>, captured Strings.
fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}

fn make_counter() -> impl FnMut() -> u32 {
    let mut c = 0;
    move || {
        c += 1;
        c
    }
}

fn make_greeter(greeting: &str) -> Box<dyn Fn(&str) -> String> {
    let g = greeting.to_string();
    Box::new(move |name| format!("{}, {}!", g, name))
}

fn compose<A, B, C>(f: impl Fn(A) -> B, g: impl Fn(B) -> C) -> impl Fn(A) -> C {
    move |x| g(f(x))
}

fn make_op(kind: &str) -> Box<dyn Fn(i32, i32) -> i32> {
    match kind {
        "add" => Box::new(|a, b| a + b),
        "mul" => Box::new(|a, b| a * b),
        _ => {
            let k = kind.len() as i32;
            Box::new(move |a, _| a * k)
        }
    }
}

fn make_suffixer(suffix: String) -> impl Fn(&str) -> String {
    move |s| {
        let mut out = s.to_string();
        out.push_str(&suffix);
        out
    }
}

fn twice<F: Fn(i32) -> i32>(f: F) -> impl Fn(i32) -> i32 {
    move |x| f(f(x))
}

fn main() {
    let add5 = make_adder(5);
    let add10 = make_adder(10);
    println!("{} {}", add5(1), add10(1));

    let mut c1 = make_counter();
    let mut c2 = make_counter();
    println!("{} {} {} {}", c1(), c1(), c2(), c1());

    let hello = make_greeter("Hello");
    let hi = make_greeter("Hi");
    println!("{} / {}", hello("Ann"), hi("Bob"));

    let f = compose(|x: i32| x * 2, |y: i32| format!("<{}>", y));
    println!("{}", f(21));
    let g = compose(make_adder(1), make_adder(100));
    println!("{}", g(0));

    for k in ["add", "mul", "other"] {
        let op = make_op(k);
        println!("{} -> {}", k, op(6, 7));
    }

    let s = make_suffixer(String::from("_v2"));
    println!("{} {}", s("file"), s("dir"));

    let quad = twice(twice(|x| x + 1));
    println!("{}", quad(0));

    let fns: Vec<(&str, Box<dyn Fn(i32) -> i32>)> = vec![
        ("double", Box::new(|x| x * 2)),
        ("square", Box::new(|x| x * x)),
        ("plus5", Box::new(make_adder(5))),
    ];
    for (name, f) in &fns {
        println!("{}(7) = {}", name, f(7));
    }
    let pipeline: i32 = fns.iter().fold(1, |acc, (_, f)| f(acc));
    println!("pipeline = {}", pipeline);
}
