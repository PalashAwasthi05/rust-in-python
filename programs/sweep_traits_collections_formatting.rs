use std::fmt;
use std::collections::{HashMap, BTreeMap, VecDeque, BinaryHeap, HashSet};
use std::cmp::{Ordering, Reverse};
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
struct P { x: f64, y: f64 }
impl std::ops::Add for P { type Output = P; fn add(self, o: P) -> P { P { x: self.x + o.x, y: self.y + o.y } } }
impl fmt::Display for P { fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "({:.1}, {:.1})", self.x, self.y) } }
trait Shape { fn area(&self) -> f64; fn name(&self) -> String { "shape".into() } }
struct Sq(f64); struct Ci(f64);
impl Shape for Sq { fn area(&self) -> f64 { self.0 * self.0 } fn name(&self) -> String { "sq".into() } }
impl Shape for Ci { fn area(&self) -> f64 { 3.14159 * self.0 * self.0 } }
fn total(shapes: &[Box<dyn Shape>]) -> f64 { shapes.iter().map(|s| s.area()).sum() }
struct Counter { n: u32 }
impl Iterator for Counter { type Item = u32; fn next(&mut self) -> Option<u32> { if self.n < 5 { self.n += 1; Some(self.n) } else { None } } }
fn largest<T: PartialOrd + Copy>(xs: &[T]) -> T { let mut m = xs[0]; for &x in xs { if x > m { m = x; } } m }
fn make_adder(k: i32) -> impl Fn(i32) -> i32 { move |x| x + k }
fn apply<F: FnMut()>(mut f: F, n: usize) { for _ in 0..n { f(); } }
fn main() {
    let p = P { x: 1.0, y: 2.5 } + P::default();
    println!("{} {:?} {:#?}", p, p, p);
    println!("[{:>8.3}] [{:<6}] [{:^7}] [{:08.2}] [{:+}] [{:x}] [{:#b}] [{:e}]", 3.14159, "ab", "mid", -2.5, 5, 255, 5, 1234.5);
    let shapes: Vec<Box<dyn Shape>> = vec![Box::new(Sq(2.0)), Box::new(Ci(1.0))];
    println!("{:.3} {} {}", total(&shapes), shapes[0].name(), shapes[1].name());
    let c: Vec<u32> = Counter { n: 0 }.zip(Counter { n: 0 }.skip(1)).map(|(a, b)| a * b).filter(|x| x % 3 == 0).collect();
    println!("{:?} {}", c, Counter { n: 0 }.sum::<u32>());
    println!("{} {} {}", largest(&[3, 7, 2]), largest(&[1.5, 0.2]), largest(&['a', 'z', 'q']));
    let add5 = make_adder(5); let mut count = 0; apply(|| count += 2, 3);
    println!("{} {}", add5(10), count);
    let mut hm: HashMap<String, Vec<i32>> = HashMap::new();
    for (i, w) in "a b a c b a".split_whitespace().enumerate() { hm.entry(w.to_string()).or_default().push(i as i32); }
    let mut keys: Vec<_> = hm.iter().collect(); keys.sort();
    println!("{:?}", keys);
    let mut bt = BTreeMap::new(); for i in 0..10 { bt.insert(i * 3, i); }
    println!("{:?}", bt.range(5..15).collect::<Vec<_>>());
    let mut dq: VecDeque<i32> = (1..=4).collect(); dq.push_front(0); dq.rotate_left(2); println!("{:?} {:?}", dq, dq.back());
    let mut h = BinaryHeap::new(); for x in [5, 1, 8, 3] { h.push(Reverse(x)); }
    let mut out = vec![]; while let Some(Reverse(x)) = h.pop() { out.push(x); } println!("{:?}", out);
    let s: HashSet<i32> = [1, 2, 3].into_iter().collect(); let t: HashSet<i32> = [2, 3, 4].into_iter().collect();
    let mut i: Vec<_> = s.intersection(&t).cloned().collect(); i.sort(); println!("{:?}", i);
    let mut v = vec![5, 3, 8, 1, 9, 3];
    v.sort_by(|a, b| b.cmp(a)); v.dedup(); println!("{:?} {:?}", v, v.binary_search_by(|x| 8.cmp(x)));
    println!("{:?}", v.windows(2).map(|w| w[0] - w[1]).collect::<Vec<_>>());
    println!("{:?}", v.chunks(4).map(|c| c.len()).collect::<Vec<_>>());
    println!("{:?}", (0..20).step_by(7).rev().collect::<Vec<_>>());
    match 3.cmp(&4) { Ordering::Less => println!("less"), _ => println!("other") }
    println!("{}", matches!(Some(3), Some(x) if x > 2));
    let r = 'outer: loop { for i in 0.. { if i == 7 { break 'outer i * 2; } } };
    println!("{}", r);
    let text = "  Hello, World  ";
    println!("[{}] {} {} {:?}", text.trim(), text.to_uppercase(), text.contains("World"), text.find('W'));
    println!("{:?}", "1,2,x,4".split(',').map(|s| s.parse::<i32>()).collect::<Vec<_>>());
    println!("{:?}", "1,2,3".split(',').map(|s| s.parse::<i32>()).collect::<Result<Vec<_>, _>>());
    println!("{} {} {:?} {:?}", 'a'.is_alphabetic(), '9'.to_digit(10).unwrap(), 'A'.to_ascii_lowercase(), "héllo".chars().rev().collect::<String>());
    println!("{:?} {:?} {} {}", 200u8.checked_add(100), 200u8.wrapping_add(100), 2i64.pow(40), (-7i32).rem_euclid(3));
    println!("{} {} {}", i32::MAX, u64::MAX, f64::EPSILON > 0.0);
    println!("{:.4} {} {}", (2.0f64).sqrt(), (7.9f64) as i32, -7 / 2);
    let arr = [[1, 2], [3, 4]]; println!("{:?} {}", arr, arr.len());
    let sl = &v[1..]; if let [first, .., last] = sl { println!("{} {}", first, last); }
    let nested = vec![vec![1], vec![2, 3]]; println!("{:?}", nested.into_iter().flatten().collect::<Vec<_>>());
    println!("{:?}", (1..=3).fold(String::new(), |acc, x| acc + &x.to_string()));
    println!("{:?}", ["b", "a"].iter().max());
    println!("{}", vec!["x", "y", "z"].join("-"));
    println!("{:?}", (1..=5).partition::<Vec<_>, _>(|x| x % 2 == 0));
    println!("{:?}", [3, 1, 2].iter().position(|&x| x == 2));
    println!("{:?}", "abc".char_indices().collect::<Vec<_>>());
    println!("{}", format!("{:?}", Some("hi")).len());
}
