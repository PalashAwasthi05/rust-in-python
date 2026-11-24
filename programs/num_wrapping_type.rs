use std::num::Wrapping;

fn main() {
    let a = Wrapping(250u8);
    let b = Wrapping(10u8);
    println!("{}", a + b);
    println!("{}", (b - a).0);
    println!("{:?}", a * b);
    let mut h = Wrapping(0u32);
    for c in "hello".bytes() {
        h = h * Wrapping(31) + Wrapping(c as u32);
    }
    println!("{}", h);
    let mut x = Wrapping(i32::MAX);
    x += Wrapping(1);
    println!("{}", x);
    x -= Wrapping(1);
    println!("{}", x.0);
    let mut y = Wrapping(3u64);
    y *= Wrapping(u64::MAX);
    println!("{}", y);
    println!("{}", -Wrapping(i8::MIN));
    println!("{}", Wrapping(1u8) << 9);
    println!("{}", !Wrapping(0u16));
    println!("{}", Wrapping(0xF0u8) & Wrapping(0x3C));
    println!("{}", Wrapping(5u8) == Wrapping(5));
    println!("{}", Wrapping(3u8) < Wrapping(4));
}
