struct Counter { n: u32 }
impl Iterator for Counter { type Item = u32; fn next(&mut self) -> Option<u32> { if self.n < 5 { self.n += 1; Some(self.n) } else { None } } }
fn main() {
    let v = vec![1, 2, 3, 4];
    let mut it = v.iter();
    println!("{:?}", it.next());
    for x in it { print!("{x} "); }
    println!();
    for x in (Counter { n: 0 }).filter(|x| *x % 2 == 1) { print!("{x} "); }
    println!();
    for x in v.iter().filter(|x| **x > 2) { print!("{x} "); }
    println!();
}
