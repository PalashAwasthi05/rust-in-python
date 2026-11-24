struct Cache<F> where F: Fn(u64) -> u64 { f: F, hits: u32 }
impl<F: Fn(u64) -> u64> Cache<F> {
    fn get(&mut self, x: u64) -> u64 { self.hits += 1; (self.f)(x) }
}
fn twice<F>(mut f: F) where F: FnMut() { f(); f(); }
fn consume<F: FnOnce() -> String>(f: F) -> String { f() }
fn compose<A, B, C>(f: impl Fn(A) -> B, g: impl Fn(B) -> C) -> impl Fn(A) -> C { move |x| g(f(x)) }
fn call_dyn(f: &dyn Fn(i32) -> i32) -> i32 { f(10) }
fn main() {
    let base = 3;
    let mut c = Cache { f: |x| x * x, hits: 0 };
    println!("{} {} {}", c.get(4), c.get(5), c.hits);
    let mut n = 0;
    twice(|| n += base);
    println!("{n}");
    let s = String::from("owned");
    println!("{}", consume(move || s));
    let h = compose(|x: i32| x + 1, |y: i32| y * 10);
    println!("{}", h(4));
    let ops: Vec<Box<dyn Fn(i32) -> i32>> = vec![Box::new(|x| x + 1), Box::new(move |x| x * base)];
    for op in ops.iter() { println!("{}", op(7)); }
    println!("{}", call_dyn(&|x| x - base));
    let add = |a: i32, b: i32| a + b;
    println!("{}", add(1, 2));
}
