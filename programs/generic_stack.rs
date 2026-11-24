#[derive(Debug)]
struct Stack<T> { items: Vec<T> }
impl<T: std::fmt::Debug> Stack<T> {
    fn new() -> Self { Stack { items: Vec::new() } }
    fn push(&mut self, t: T) { self.items.push(t); }
    fn pop(&mut self) -> Option<T> { self.items.pop() }
    fn peek(&self) -> Option<&T> { self.items.last() }
    fn len(&self) -> usize { self.items.len() }
    fn is_empty(&self) -> bool { self.items.is_empty() }
}
fn largest<T: PartialOrd + Copy>(xs: &[T]) -> T { let mut m = xs[0]; for &x in xs { if x > m { m = x; } } m }
fn main() { let mut s = Stack::new(); s.push(1); s.push(2); println!("{:?}", s.peek()); println!("{:?} {} {}", s.pop(), s.len(), s.is_empty()); println!("{} {}", largest(&[1, 5, 3]), largest(&[1.5, 0.2])); }
