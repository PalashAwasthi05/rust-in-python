trait Iter {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
    fn count2(mut self) -> usize where Self: Sized { let mut n = 0; while let Some(_) = self.next() { n += 1; } n }
    fn take2(self, n: usize) -> Take2<Self> where Self: Sized { Take2 { inner: self, n: n } }
    fn map2<B>(self, f: fn(Self::Item) -> B) -> Map2<Self, B> where Self: Sized { Map2 { inner: self, f: f } }
}
struct Take2<I> { inner: I, n: usize }
impl<I: Iter> Iter for Take2<I> {
    type Item = I::Item;
    fn next(&mut self) -> Option<I::Item> { if self.n == 0 { None } else { self.n -= 1; self.inner.next() } }
}
struct Map2<I: Iter, B> { inner: I, f: fn(I::Item) -> B }
impl<I: Iter, B> Iter for Map2<I, B> {
    type Item = B;
    fn next(&mut self) -> Option<B> { match self.inner.next() { Some(x) => Some((self.f)(x)), None => None } }
}
struct Up { n: u32 }
impl Iter for Up { type Item = u32; fn next(&mut self) -> Option<u32> { self.n += 1; Some(self.n) } }
fn dbl(x: u32) -> u64 { (x * 2) as u64 }
fn main() {
    let mut t = Up { n: 0 }.take2(3).map2(dbl);
    while let Some(x) = t.next() { println!("{x}"); }
    println!("{}", Up { n: 0 }.take2(4).count2());
}
