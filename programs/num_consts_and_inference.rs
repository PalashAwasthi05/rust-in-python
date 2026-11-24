const MASK: u32 = (1 << 20) - 1;
const BIG: u64 = 1 << 40;
const NEG: i16 = -(1 << 14);
const KB: usize = 1024;
const SIZES: [usize; 3] = [KB, KB * KB, KB * KB * KB];
const SCALE: f64 = 1.0 / 3.0;
const MIX: u64 = 0x9E37_79B9_7F4A_7C15;
static LIMIT: i32 = i32::MAX - 10;
const HALF: u8 = u8::MAX / 2;
const SHIFTED: i64 = -1i64 << 40;
const CAST: u8 = 300u32 as u8;
const FCAST: i32 = 3.9e9 as i32;

fn clamp_to_u8(x: i64) -> u8 {
    if x < 0 { 0 } else if x > 255 { 255 } else { x as u8 }
}

fn checksum(data: &[u8]) -> u8 {
    data.iter().fold(0u8, |acc, &b| acc.wrapping_add(b))
}

fn luhn(number: &str) -> bool {
    let mut sum = 0;
    for (i, c) in number.chars().rev().enumerate() {
        let mut d = c.to_digit(10).unwrap();
        if i % 2 == 1 {
            d *= 2;
            if d > 9 { d -= 9; }
        }
        sum += d;
    }
    sum % 10 == 0
}

fn gray(n: u32) -> u32 { n ^ (n >> 1) }

fn main() {
    println!("{} {} {} {:?}", MASK, BIG, NEG, SIZES);
    println!("{} {} {}", SCALE, MIX, LIMIT);
    println!("{} {} {} {}", HALF, SHIFTED, CAST, FCAST);
    println!("{} {} {}", clamp_to_u8(-3), clamp_to_u8(77), clamp_to_u8(1000));
    println!("{}", checksum(&[200, 100, 50, 25]));
    println!("{} {}", luhn("79927398713"), luhn("79927398710"));
    let grays: Vec<String> = (0..8).map(|n| format!("{:03b}", gray(n))).collect();
    println!("{}", grays.join(" "));
    let x = 5;
    let y = x * 1_000_000_000_000i64;
    println!("{}", y);
    let mut acc = 0;
    for i in 0..10u64 { acc += i * i; }
    println!("{}", acc);
    let v = vec![1, 2, 3];
    let n: i32 = v.iter().sum();
    println!("{}", n);
    let idx = v.len() - 1;
    println!("{}", v[idx]);
    let half = v.len() / 2;
    println!("{}", half);
    let avg = v.iter().sum::<i32>() as f64 / v.len() as f64;
    println!("{}", avg);
    let bits = 0u16;
    let flipped = !bits;
    println!("{}", flipped);
    let a = [1.5, 2.5];
    let total: f32 = a.iter().sum();
    println!("{}", total);
    let t = (1 << 31) as u32;
    println!("{}", t);
    let mut m = [[0i8; 3]; 2];
    m[1][2] = -1;
    println!("{:?}", m);
    let z = -5 % 3;
    println!("{}", z);
    let q = 7 / -2;
    println!("{}", q);
    let p = 2i32.pow(30) + (2i32.pow(30) - 1);
    println!("{}", p);
    let e = 10_f64.powi(-3);
    println!("{}", e);
    let w = u32::MAX as usize + 1;
    println!("{}", w);
    let small = 1u8.checked_sub(2).map_or(-1, |v| v as i32);
    println!("{}", small);
    let r = (1..=20u64).fold(1, |a, b| a * b / gcd(a, b));
    println!("{}", r);
    let h = (0..64).map(|i| (i as u64) << i % 7).fold(0u64, |a, b| a ^ b);
    println!("{}", h);
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}
