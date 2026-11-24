use std::fmt::Display;

fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}

fn compose<A, B, C>(f: impl Fn(A) -> B, g: impl Fn(B) -> C) -> impl Fn(A) -> C {
    move |x| g(f(x))
}

fn squares(limit: u32) -> impl Iterator<Item = u32> {
    (1..).map(|x| x * x).take_while(move |&x| x <= limit)
}

fn words<'a>(s: &'a str) -> impl Iterator<Item = String> + 'a {
    s.split_whitespace().filter(|w| w.len() > 2).map(|w| w.to_lowercase())
}

fn join_all(items: impl IntoIterator<Item = impl Display>) -> String {
    items.into_iter().map(|x| format!("<{}>", x)).collect::<Vec<_>>().join("")
}

fn apply_n(f: &impl Fn(i64) -> i64, n: usize, x: i64) -> i64 {
    (0..n).fold(x, |acc, _| f(acc))
}

fn counter() -> impl FnMut() -> u32 {
    let mut c = 0;
    move || {
        c += 1;
        c
    }
}

fn describe(x: impl Display + PartialOrd + Copy, pivot: impl Into<f64>) -> String {
    format!("{} vs {}", x, pivot.into())
}

fn pairs<T: Clone>(v: &[T]) -> impl Iterator<Item = (T, T)> + '_ {
    v.iter().enumerate().flat_map(move |(i, a)| v[i + 1..].iter().map(move |b| (a.clone(), b.clone())))
}

fn boxed_ops() -> Vec<Box<dyn Fn(i32) -> i32>> {
    vec![Box::new(|x| x + 1), Box::new(|x| x * 10), Box::new(make_adder(-5))]
}

fn main() {
    let add5 = make_adder(5);
    println!("{}", add5(10));
    let f = compose(add5, |x: i32| x * 2);
    println!("{}", f(1));
    let g = compose(|s: &str| s.len(), |n: usize| format!("len={}", n));
    println!("{}", g("hello"));
    println!("{:?}", squares(50).collect::<Vec<_>>());
    println!("{:?}", words("The quick Brown fox is OK").collect::<Vec<_>>());
    println!("{}", join_all(vec![1, 2, 3]));
    println!("{}", join_all(["x", "y"]));
    let double = |x| x * 2;
    println!("{}", apply_n(&double, 10, 1));
    let mut next = counter();
    next();
    next();
    println!("counter at {}", next());
    println!("{}", describe(3u8, 2.5f32));
    println!("{:?}", pairs(&['a', 'b', 'c']).collect::<Vec<_>>());
    let ops = boxed_ops();
    println!("{:?}", ops.iter().map(|op| op(7)).collect::<Vec<_>>());
    let total: i32 = ops.iter().fold(1, |acc, op| op(acc));
    println!("{}", total);
}
