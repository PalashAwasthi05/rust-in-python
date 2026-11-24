fn apply<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 { f(x) }
fn make_adder(n: i32) -> impl Fn(i32) -> i32 { move |x| x + n }
fn boxed() -> Box<dyn Fn(i32) -> i32> { Box::new(|x| x * 2) }
fn main() {
    let mut count = 0;
    let mut inc = || count += 1;
    inc(); inc();
    println!("{} {} {} {}", count, apply(|x| x * 3, 4), make_adder(5)(1), boxed()(7));
}
