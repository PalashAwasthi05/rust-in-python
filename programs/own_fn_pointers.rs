// Function pointers, recursion through them, and higher-order functions.
fn fact(n: u64) -> u64 {
    if n == 0 { 1 } else { n * fact(n - 1) }
}

fn fib(n: u64) -> u64 {
    if n < 2 { n } else { fib(n - 1) + fib(n - 2) }
}

fn apply(f: fn(u64) -> u64, xs: &[u64]) -> Vec<u64> {
    xs.iter().map(|&x| f(x)).collect()
}

fn fix(f: &dyn Fn(&dyn Fn(u32) -> u32, u32) -> u32, n: u32) -> u32 {
    f(&|k| fix(f, k), n)
}

fn map_all<F>(v: &mut [i32], f: F)
where
    F: Fn(i32) -> i32,
{
    for x in v.iter_mut() {
        *x = f(*x);
    }
}

fn find_first<T, P: Fn(&T) -> bool>(items: &[T], pred: P) -> Option<&T> {
    for it in items {
        if pred(it) {
            return Some(it);
        }
    }
    None
}

fn is_even(n: i32) -> bool {
    n % 2 == 0
}

struct Op {
    name: &'static str,
    run: fn(i32) -> i32,
}

fn neg(x: i32) -> i32 {
    -x
}

fn walk(depth: u32, visit: &mut dyn FnMut(u32)) {
    visit(depth);
    if depth > 0 {
        walk(depth - 1, visit);
    }
}

fn main() {
    let table: [(&str, fn(u64) -> u64); 2] = [("fact", fact), ("fib", fib)];
    for (name, f) in table.iter() {
        println!("{}: {:?}", name, apply(*f, &[0, 1, 5, 10]));
    }

    let tri = fix(&|rec, n| if n == 0 { 0 } else { n + rec(n - 1) }, 10);
    println!("tri(10) = {}", tri);

    let mut data = vec![1, 2, 3, 4];
    let offset = 100;
    map_all(&mut data, |x| x * x + offset);
    println!("{:?}", data);
    map_all(&mut data, neg);
    println!("{:?}", data);

    let nums = [3, 7, 8, 11, 12];
    println!("{:?}", find_first(&nums, |&n| is_even(n)));
    println!("{:?}", find_first(&nums, |&n| n > 100));
    let words = ["apple", "kiwi", "banana"];
    println!("{:?}", find_first(&words, |w| w.len() == 4));

    let ops = [
        Op { name: "neg", run: neg },
        Op { name: "dbl", run: |x| x * 2 },
        Op { name: "abs", run: i32::abs },
    ];
    for op in &ops {
        println!("{} -> {}", op.name, (op.run)(-21));
    }
    let evens: Vec<i32> = (1..10).filter(|&n| is_even(n)).collect();
    println!("{:?}", evens);
    let strs: Vec<String> = evens.iter().map(ToString::to_string).collect();
    println!("{}", strs.join(","));

    let mut seen = Vec::new();
    walk(3, &mut |d| seen.push(d * 10));
    println!("{:?}", seen);
    let f: fn(i32) -> bool = is_even;
    let g = f;
    println!("{} {}", f(4), g(5));
}
