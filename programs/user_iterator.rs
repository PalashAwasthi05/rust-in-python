struct Counter { n: u32 }
impl Iterator for Counter { type Item = u32; fn next(&mut self) -> Option<u32> { if self.n < 5 { self.n += 1; Some(self.n) } else { None } } }
fn main() {
    let s: u32 = Counter { n: 0 }.zip(Counter { n: 0 }.skip(1)).map(|(a, b)| a * b).filter(|x| x % 3 == 0).sum();
    println!("{s}");
    for x in (Counter { n: 2 }) { print!("{x} "); }
    println!();
}
