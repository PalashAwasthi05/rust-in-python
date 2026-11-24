use std::fmt;
use std::ops::{Add, Mul, Neg, Sub, AddAssign, Index, IndexMut};

#[derive(Debug, Clone, Copy, PartialEq)]
struct V2 { x: f64, y: f64 }
impl Add for V2 { type Output = V2; fn add(self, o: V2) -> V2 { V2 { x: self.x + o.x, y: self.y + o.y } } }
impl Sub for V2 { type Output = V2; fn sub(self, o: V2) -> V2 { V2 { x: self.x - o.x, y: self.y - o.y } } }
impl Mul<f64> for V2 { type Output = V2; fn mul(self, k: f64) -> V2 { V2 { x: self.x * k, y: self.y * k } } }
impl Mul for V2 { type Output = f64; fn mul(self, o: V2) -> f64 { self.x * o.x + self.y * o.y } }
impl Neg for V2 { type Output = V2; fn neg(self) -> V2 { V2 { x: -self.x, y: -self.y } } }
impl AddAssign for V2 { fn add_assign(&mut self, o: V2) { self.x += o.x; self.y += o.y; } }

struct Grid { w: usize, cells: Vec<u8> }
impl Index<(usize, usize)> for Grid { type Output = u8; fn index(&self, (r, c): (usize, usize)) -> &u8 { &self.cells[r * self.w + c] } }
impl IndexMut<(usize, usize)> for Grid { fn index_mut(&mut self, (r, c): (usize, usize)) -> &mut u8 { &mut self.cells[r * self.w + c] } }

#[derive(Debug)]
enum AppError { Parse(std::num::ParseIntError), Negative(i64) }
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self { AppError::Parse(e) => write!(f, "parse error: {}", e), AppError::Negative(n) => write!(f, "{} is negative", n) }
    }
}
impl std::error::Error for AppError {}
impl From<std::num::ParseIntError> for AppError { fn from(e: std::num::ParseIntError) -> Self { AppError::Parse(e) } }

fn parse_positive(s: &str) -> Result<i64, AppError> {
    let n: i64 = s.trim().parse()?;
    if n < 0 { return Err(AppError::Negative(n)); }
    Ok(n)
}

fn run(s: &str) -> Result<i64, Box<dyn std::error::Error>> {
    let n = parse_positive(s)?;
    if n == 0 { return Err("zero is not allowed".into()); }
    Ok(n * 2)
}

struct Meters(f64);
impl From<f64> for Meters { fn from(v: f64) -> Self { Meters(v) } }

fn main() {
    let a = V2 { x: 1.0, y: 2.0 };
    let b = V2 { x: 3.0, y: 4.0 };
    let mut c = a + b * 2.0 - -a;
    c += a;
    println!("{:?} {} {}", c, a * b, a == V2 { x: 1.0, y: 2.0 });
    let mut g = Grid { w: 3, cells: vec![0; 9] };
    g[(1, 2)] = 7;
    g[(0, 0)] += 1;
    println!("{} {} {:?}", g[(1, 2)], g[(0, 0)], g.cells);
    for s in ["21", "x", "-4", "0"] {
        match run(s) { Ok(v) => println!("ok {v}"), Err(e) => println!("error: {e}") }
    }
    let s: String = "abc".into();
    let n: i64 = 7i32.into();
    let m: Meters = 2.5.into();
    let small: Result<u8, _> = 300i32.try_into();
    let ok: u8 = 200i32.try_into().unwrap();
    println!("{s} {n} {} {:?} {}", m.0, small.is_err(), ok);
    let big = u64::from(3u32) + i64::from(-2i8).unsigned_abs();
    println!("{} {}", big, String::from('x'));
}
