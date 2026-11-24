use std::num::Wrapping;
use std::ops::{Shl, Shr, BitAnd, BitOr, Not, Neg};

#[derive(Debug, Clone, Copy, PartialEq)]
struct Flags(u8);
impl BitOr for Flags { type Output = Flags; fn bitor(self, o: Flags) -> Flags { Flags(self.0 | o.0) } }
impl BitAnd for Flags { type Output = Flags; fn bitand(self, o: Flags) -> Flags { Flags(self.0 & o.0) } }
impl Not for Flags { type Output = Flags; fn not(self) -> Flags { Flags(!self.0) } }
impl Shl<u32> for Flags { type Output = Flags; fn shl(self, n: u32) -> Flags { Flags(self.0 << n) } }
impl Shr<u32> for Flags { type Output = Flags; fn shr(self, n: u32) -> Flags { Flags(self.0 >> n) } }
impl Neg for Flags { type Output = Flags; fn neg(self) -> Flags { Flags(self.0.wrapping_neg()) } }

fn main() {
    let a = Flags(0b0011);
    println!("{:?} {:?} {:?}", a | Flags(0b0100), a & Flags(1), !a);
    println!("{:?} {:?} {:?}", a << 2, a >> 1, -a);
    let mut f = a;
    f = f << 1;
    println!("{:?}", f);
    let x: u64 = 0xFFFF_FFFF;
    let shifted = x << 16u8;
    println!("{}", shifted);
    let s: i64 = -1;
    println!("{}", s >> 3u32);
    let k: u8 = 3;
    println!("{}", 1u64 << k);
    println!("{}", 1i32 << k as i64);
    let mut w = Wrapping(1u32);
    w <<= 31;
    println!("{}", w);
    w = w >> 4;
    println!("{}", w);
    println!("{}", Wrapping(7i32) / Wrapping(2));
    println!("{}", Wrapping(-7i32) % Wrapping(3));
    println!("{:?}", Wrapping(i32::MIN) / Wrapping(-1));
    let mut h = Wrapping(0xcbf29ce484222325u64);
    for b in b"abc" {
        h ^= Wrapping(*b as u64);
        h *= Wrapping(0x100000001b3);
    }
    println!("{:x}", h.0);
    println!("{}", (i128::MAX as f64) as i128);
    println!("{}", (u128::MAX as f32) as u128);
    println!("{}", -1.5f64 as u128);
    println!("{}", 1e30f32 as i128);
    println!("{}", (0.1f32 + 0.2f32) as f64);
    println!("{}", 1u64 as f32 / 3.0);
    let big = 1u128 << 100;
    println!("{}", big as f64);
    println!("{}", big as f32);
    println!("{}", -(big as i128));
    println!("{}", i16::MAX as i8);
    println!("{}", 0x1_0000_0001u64 as u32);
    println!("{}", -0.0f64 as i32);
    println!("{:?}", (2.0f64).powi(i32::MIN));
    println!("{}", 10f64.powf(-2.0));
    println!("{}", (-8.0f64).cbrt());
    println!("{}", (-8.0f64).powf(1.0 / 3.0));
    println!("{}", 0.0f64.ln());
    println!("{}", 1e308f64 * 10.0);
    println!("{}", (1.0f64).exp_m1());
    println!("{}", 1e-10f64.ln_1p());
    println!("{}", f64::consts_check());
}

trait C { fn consts_check() -> f64; }
impl C for f64 { fn consts_check() -> f64 { std::f64::consts::TAU + std::f64::consts::SQRT_2 + std::f64::consts::LN_2 } }
