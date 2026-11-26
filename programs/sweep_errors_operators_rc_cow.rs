use std::rc::{Rc, Weak};
use std::cell::{RefCell, Cell};
use std::collections::HashSet;
use std::fmt;
use std::borrow::Cow;
use std::ops::{Index, IndexMut, Neg, AddAssign, Mul};
#[derive(Debug)]
enum MyErr { Parse(std::num::ParseIntError), Neg(i64) }
impl From<std::num::ParseIntError> for MyErr { fn from(e: std::num::ParseIntError) -> Self { MyErr::Parse(e) } }
impl fmt::Display for MyErr { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { match self { MyErr::Parse(e) => write!(f, "parse: {}", e), MyErr::Neg(n) => write!(f, "neg {}", n) } } }
impl std::error::Error for MyErr {}
fn parse_pos(s: &str) -> Result<i64, MyErr> { let n: i64 = s.trim().parse()?; if n < 0 { return Err(MyErr::Neg(n)); } Ok(n) }
fn run(s: &str) -> Result<i64, Box<dyn std::error::Error>> { let v = parse_pos(s)?; Ok(v * 2) }
trait Animal { const LEGS: u32; type Food: fmt::Debug; fn food(&self) -> Self::Food; fn describe<T: fmt::Display>(&self, t: T) -> String { format!("{} legs, {}", Self::LEGS, t) } }
struct Dog;
impl Animal for Dog { const LEGS: u32 = 4; type Food = &'static str; fn food(&self) -> &'static str { "bone" } }
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Pt { x: i32, y: i32 }
struct Grid { w: usize, cells: Vec<char> }
impl Index<(usize, usize)> for Grid { type Output = char; fn index(&self, (r, c): (usize, usize)) -> &char { &self.cells[r * self.w + c] } }
impl IndexMut<(usize, usize)> for Grid { fn index_mut(&mut self, (r, c): (usize, usize)) -> &mut char { &mut self.cells[r * self.w + c] } }
#[derive(Debug, Clone, Copy)] struct V2(i32, i32);
impl Neg for V2 { type Output = V2; fn neg(self) -> V2 { V2(-self.0, -self.1) } }
impl AddAssign for V2 { fn add_assign(&mut self, o: V2) { self.0 += o.0; self.1 += o.1; } }
impl Mul<i32> for V2 { type Output = V2; fn mul(self, k: i32) -> V2 { V2(self.0 * k, self.1 * k) } }
#[derive(Debug)] struct TreeNode { val: i32, children: RefCell<Vec<Rc<TreeNode>>>, parent: RefCell<Weak<TreeNode>> }
struct Parser<'a> { src: &'a str, pos: usize }
impl<'a> Parser<'a> { fn word(&mut self) -> Option<&'a str> { let rest = &self.src[self.pos..]; let rest_t = rest.trim_start(); if rest_t.is_empty() { return None; } let start = self.src.len() - rest_t.len(); let end = rest_t.find(' ').map(|i| start + i).unwrap_or(self.src.len()); self.pos = end; Some(&self.src[start..end]) } }
fn normalize(s: &str) -> Cow<str> { if s.contains(' ') { Cow::Owned(s.replace(' ', "_")) } else { Cow::Borrowed(s) } }
const fn sq(x: u32) -> u32 { x * x }
static GREETING: &str = "hey";
const N: u32 = sq(4);
fn compose<A, B, C>(f: impl Fn(A) -> B, g: impl Fn(B) -> C) -> impl Fn(A) -> C { move |x| g(f(x)) }
fn main() {
    for s in ["21", " 4 ", "-3", "x"] { match run(s) { Ok(v) => println!("ok {}", v), Err(e) => println!("err {}", e) } }
    println!("{:?}", parse_pos("q"));
    let d = Dog; println!("{} {:?} {}", Dog::LEGS, d.food(), d.describe(1.5));
    let mut set = HashSet::new(); set.insert(Pt { x: 1, y: 2 }); set.insert(Pt { x: 1, y: 2 }); set.insert(Pt { x: 0, y: 5 });
    let mut pts: Vec<_> = set.into_iter().collect(); pts.sort(); println!("{:?}", pts);
    let mut g = Grid { w: 3, cells: vec!['.'; 9] }; g[(1, 2)] = '#'; println!("{} {}", g[(1, 2)], g.cells.iter().collect::<String>());
    let mut a = V2(1, 2); a += -V2(3, 3) * 2; println!("{:?}", a);
    let leaf = Rc::new(TreeNode { val: 3, children: RefCell::new(vec![]), parent: RefCell::new(Weak::new()) });
    let branch = Rc::new(TreeNode { val: 5, children: RefCell::new(vec![Rc::clone(&leaf)]), parent: RefCell::new(Weak::new()) });
    *leaf.parent.borrow_mut() = Rc::downgrade(&branch);
    println!("{:?} {} {} {}", leaf.parent.borrow().upgrade().map(|p| p.val), Rc::strong_count(&leaf), Rc::weak_count(&branch), branch.children.borrow().len());
    let c = Cell::new(5); c.set(c.get() + 1); println!("{}", c.get());
    let shared = Rc::new(RefCell::new(vec![1])); let s2 = Rc::clone(&shared); s2.borrow_mut().push(2); println!("{:?}", shared.borrow());
    let mut p = Parser { src: "alpha beta  gamma", pos: 0 }; let mut ws = vec![]; while let Some(w) = p.word() { ws.push(w); } println!("{:?}", ws);
    println!("{} {}", normalize("a b"), normalize("ab"));
    println!("{} {} {}", N, GREETING, compose(|x: i32| x + 1, |y: i32| y * 10)(4));
    let width = 7; println!("[{:>width$}] [{:>1$}] [{:.*}]", "r", 5, 2, 1.23456);
    let mut v = vec![1, 2, 3]; let taken = std::mem::take(&mut v); let old = std::mem::replace(&mut v, vec![9]); println!("{:?} {:?} {:?}", taken, old, v);
    let mut it = (1..10).peekable(); let mut groups = vec![]; while let Some(x) = it.next() { if let Some(&n) = it.peek() { if n % 3 == 0 { groups.push(x); } } } println!("{:?}", groups);
    let mut it2 = 1..10; let first3: Vec<_> = it2.by_ref().take(3).collect(); println!("{:?} {:?}", first3, it2.next());
    println!("{:?} {:?}", Some(4).and_then(|x| if x > 3 { Some(x * 2) } else { None }).filter(|x| *x > 5), Some(1).zip(Some("a")));
    let mut fl = vec![2.5, -1.0, 3.75]; fl.sort_by(|a, b| a.partial_cmp(b).unwrap()); println!("{:?}", fl);
    let msg = match 15 { n @ 1..=9 => format!("small {}", n), n @ (10 | 15) => format!("ten-ish {}", n), _ => "big".into() }; println!("{}", msg);
    println!("{}", (b'a' + 2) as char);
    println!("{:?}", 1e21f64); println!("{:?} {} {}", 0.1 + 0.2, 1.0f64, f64::NAN);
    let words = vec!["apple", "bob", "kayak"]; let pals: Vec<_> = words.iter().filter(|w| w.chars().eq(w.chars().rev())).collect(); println!("{:?}", pals);
    let v2: Vec<i32> = Vec::with_capacity(10); println!("{} {}", v2.len(), v2.is_empty());
    println!("{:?}", i32::from_str_radix("-ff", 16));
    println!("{}", u8::try_from(300i32).is_err());
    let x: i32 = 2147483600; let y = x.saturating_add(100); println!("{}", y);
    std::process::exit(match y { 2147483647 => 3, _ => 0 });
}
