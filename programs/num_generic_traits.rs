use std::fmt::Display;
use std::ops::{Add, Mul, Sub};

trait Bits {
    const WIDTH: u32;
    fn ones(self) -> u32;
}

macro_rules! impl_bits {
    ($($t:ty),*) => {
        $(impl Bits for $t {
            const WIDTH: u32 = <$t>::BITS;
            fn ones(self) -> u32 { self.count_ones() }
        })*
    };
}

impl_bits!(u8, u16, u32, u64, i8, i32, i64);

fn report<T: Bits + Copy + Display>(x: T) {
    println!("{} has {} ones of {}", x, x.ones(), T::WIDTH);
}

trait Zero { fn zero() -> Self; }
impl Zero for i32 { fn zero() -> Self { 0 } }
impl Zero for f64 { fn zero() -> Self { 0.0 } }
impl Zero for u64 { fn zero() -> Self { 0 } }

fn dot<T: Zero + Copy + Add<Output = T> + Mul<Output = T>>(a: &[T], b: &[T]) -> T {
    let mut acc = T::zero();
    for i in 0..a.len() {
        acc = acc + a[i] * b[i];
    }
    acc
}

fn diff<T: Sub<Output = T> + PartialOrd + Copy>(a: T, b: T) -> T {
    if a > b { a - b } else { b - a }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Money(i64);

impl Add for Money {
    type Output = Money;
    fn add(self, o: Money) -> Money { Money(self.0 + o.0) }
}

impl Mul<i64> for Money {
    type Output = Money;
    fn mul(self, k: i64) -> Money { Money(self.0 * k) }
}

impl std::fmt::Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let sign = if self.0 < 0 { "-" } else { "" };
        write!(f, "{}${}.{:02}", sign, self.0.abs() / 100, self.0.abs() % 100)
    }
}

fn mean<T: Into<f64> + Copy>(xs: &[T]) -> f64 {
    let mut total = 0.0;
    for &x in xs {
        total += x.into();
    }
    total / xs.len() as f64
}

fn to_u8_saturating(x: i32) -> u8 {
    x.clamp(0, 255) as u8
}

fn main() {
    report(200u8);
    report(-1i8);
    report(0xFFFFu16);
    report(u64::MAX);
    report(-2i64);
    println!("{}", dot(&[1, 2, 3], &[4, 5, 6]));
    println!("{}", dot(&[0.5, 1.5], &[2.0, 4.0]));
    println!("{}", dot(&[u64::MAX / 4, 1], &[2, 3]));
    println!("{} {} {}", diff(3, 10), diff(2.5, 1.0), diff(7u8, 200u8));
    let price = Money(1999);
    let total = price * 3 + Money(-50);
    println!("{} {:?}", total, total);
    println!("{}", Money(-705));
    println!("{}", mean(&[1u8, 2, 4]));
    println!("{}", mean(&[1.5f32, 2.25]));
    println!("{}", mean(&[-3i32, 3, 9]));
    println!("{} {} {}", to_u8_saturating(-5), to_u8_saturating(128), to_u8_saturating(999));
    let values = [3i64, -7, 12, 0];
    let total: i64 = values.iter().map(|v| v.pow(2)).sum();
    println!("{}", total);
    let max_abs = values.iter().map(|v| v.abs()).max().unwrap();
    println!("{}", max_abs);
    let bytes: Vec<u8> = values.iter().map(|&v| v as u8).collect();
    println!("{:?}", bytes);
    let floats: Vec<f64> = values.iter().map(|&v| v as f64 / 4.0).collect();
    println!("{:?}", floats);
    let parsed: Vec<u16> = "10 20 300 4000".split(' ').map(|s| s.parse().unwrap()).collect();
    println!("{:?} {}", parsed, parsed.iter().map(|&x| x as u32).sum::<u32>());
    let mut sorted = vec![2.5f64, -1.0, 9.75, 0.0, -3.5];
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!("{:?}", sorted);
    sorted.sort_by(|a, b| b.total_cmp(a));
    println!("{:?}", sorted);
    let m = sorted.iter().cloned().fold(f64::NAN, f64::min);
    println!("{}", m);
    let counts = [0u32; 3].iter().enumerate().map(|(i, _)| i as u32 * 10).collect::<Vec<_>>();
    println!("{:?}", counts);
}
