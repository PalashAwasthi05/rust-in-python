use std::fmt;
use std::ops::{Add, AddAssign, Index, IndexMut, Mul, Neg, Not, Sub};

#[derive(Debug, Clone, Copy, PartialEq)]
struct Vec2 { x: f64, y: f64 }

impl Add for Vec2 { type Output = Vec2; fn add(self, o: Vec2) -> Vec2 { Vec2 { x: self.x + o.x, y: self.y + o.y } } }
impl Sub for Vec2 { type Output = Vec2; fn sub(self, o: Vec2) -> Vec2 { Vec2 { x: self.x - o.x, y: self.y - o.y } } }
impl Mul<f64> for Vec2 { type Output = Vec2; fn mul(self, k: f64) -> Vec2 { Vec2 { x: self.x * k, y: self.y * k } } }
impl Mul<Vec2> for f64 { type Output = Vec2; fn mul(self, v: Vec2) -> Vec2 { v * self } }
impl Mul for Vec2 { type Output = f64; fn mul(self, o: Vec2) -> f64 { self.x * o.x + self.y * o.y } }
impl Neg for Vec2 { type Output = Vec2; fn neg(self) -> Vec2 { Vec2 { x: -self.x, y: -self.y } } }
impl AddAssign for Vec2 { fn add_assign(&mut self, o: Vec2) { self.x += o.x; self.y += o.y; } }

impl fmt::Display for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "({:.2}, {:.2})", self.x, self.y) }
}

struct Foo;
struct Bar;
#[derive(Debug)]
struct FooBar;
#[derive(Debug)]
struct BarFoo;

impl Add<Bar> for Foo {
    type Output = FooBar;
    fn add(self, _rhs: Bar) -> FooBar {
        println!("> Foo.add(Bar) was called");
        FooBar
    }
}

impl Add<Foo> for Bar {
    type Output = BarFoo;
    fn add(self, _rhs: Foo) -> BarFoo {
        println!("> Bar.add(Foo) was called");
        BarFoo
    }
}

struct Polynomial(Vec<i64>);

impl Index<usize> for Polynomial {
    type Output = i64;
    fn index(&self, i: usize) -> &i64 { self.0.get(i).unwrap_or(&0) }
}

impl IndexMut<usize> for Polynomial {
    fn index_mut(&mut self, i: usize) -> &mut i64 {
        if i >= self.0.len() { self.0.resize(i + 1, 0); }
        &mut self.0[i]
    }
}

impl<'a> Add for &'a Polynomial {
    type Output = Polynomial;
    fn add(self, o: &Polynomial) -> Polynomial {
        let n = self.0.len().max(o.0.len());
        Polynomial((0..n).map(|i| self[i] + o[i]).collect())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Flags(u8);
impl Not for Flags { type Output = Flags; fn not(self) -> Flags { Flags(!self.0) } }

fn main() {
    let a = Vec2 { x: 1.0, y: 2.0 };
    let b = Vec2 { x: 0.5, y: -1.5 };
    println!("a + b = {}", a + b);
    println!("a - b = {}", a - b);
    println!("a * 3 = {}, 2 * b = {}", a * 3.0, 2.0 * b);
    println!("dot(a, b) = {}", a * b);
    println!("-a = {}", -a);
    let mut pos = Vec2 { x: 0.0, y: 0.0 };
    for _ in 0..4 { pos += b; }
    println!("after 4 steps: {}", pos);
    println!("equal: {}", a + b == b + a);

    println!("Foo + Bar = {:?}", Foo + Bar);
    println!("Bar + Foo = {:?}", Bar + Foo);

    let mut p = Polynomial(vec![1, 2]);
    p[4] = 7;
    let q = Polynomial(vec![0, 0, 3]);
    let sum = &p + &q;
    println!("p = {:?}, p[10] = {}, sum = {:?}", p.0, p[10], sum.0);
    println!("!Flags(0b1010_0000) = {:?}", !Flags(0b1010_0000));
}
