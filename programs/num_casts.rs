fn main() {
    // integer truncation and sign extension
    let a: i32 = -1;
    println!("{} {} {} {}", a as u8, a as u16, a as u32, a as u64);
    println!("{} {}", a as u128, a as usize);
    let b: i64 = 300;
    println!("{} {} {}", b as u8, b as i8, b as i16);
    let c: u8 = 200;
    println!("{} {} {} {}", c as i8, c as i32, c as i64, c as u64);
    let d: i8 = -128;
    println!("{} {} {} {}", d as u8, d as i16, d as u16, d as i64);
    let e: u64 = u64::MAX;
    println!("{} {} {} {}", e as i64, e as i32, e as u32, e as i128);
    let f: i128 = -170141183460469231731687303715884105728;
    println!("{} {} {}", f as u128, f as i64, f as u8);
    let g: u32 = 0xDEADBEEF;
    println!("{} {} {} {}", g as i32, g as u16, g as i16, g as u8);
    println!("{}", 65537u32 as u16);
    println!("{}", (-32769i32) as i16);
    // float -> int saturating
    println!("{} {} {}", 3.99f64 as i32, -3.99f64 as i32, 300.0f64 as u8);
    println!("{} {} {}", -5.0f64 as u8, f64::NAN as i32, f64::INFINITY as i64);
    println!("{} {}", f64::NEG_INFINITY as i64, 1e20f64 as i64);
    println!("{} {}", -1e20f64 as i64, 1e40f64 as u128);
    println!("{} {}", 2.5f32 as u8, -0.9f32 as i8);
    println!("{} {}", 255.9f64 as u8, 256.0f64 as u8);
    println!("{}", f32::MAX as u64);
    // int -> float rounding
    println!("{}", u64::MAX as f64);
    println!("{}", u64::MAX as f32);
    println!("{}", 16777217i32 as f32);
    println!("{}", 9007199254740993i64 as f64);
    println!("{}", i64::MIN as f64);
    println!("{}", 123456789u32 as f32);
    println!("{}", u128::MAX as f64);
    // f64 <-> f32
    println!("{}", 0.1f64 as f32);
    println!("{}", 0.1f32 as f64);
    println!("{}", 1e300f64 as f32);
    println!("{}", (1.0f64 / 3.0) as f32);
    // char / bool
    println!("{}", 97u8 as char);
    println!("{}", 'A' as u32);
    println!("{}", 'é' as u8);
    println!("{}", '€' as u32);
    println!("{}", '€' as u8);
    println!("{} {}", true as i32, false as u8);
    println!("{}", true as u64 + 41);
    let ch = (b'a' + 3) as char;
    println!("{}", ch);
    let digits: String = (0..10u8).map(|d| (b'0' + d) as char).collect();
    println!("{}", digits);
    println!("{}", ('a' as u8 + 25) as char);
    println!("{:?}", char::from_u32(0x1F600));
    println!("{:?}", char::from_digit(7, 10));
    println!("{:?}", 'z'.to_digit(36));
    // isize/usize
    let idx: usize = 5;
    let off: isize = -7;
    println!("{}", (idx as isize + off) as usize);
    println!("{}", usize::MAX);
    println!("{}", isize::MIN);
    println!("{}", (-1isize) as usize);
    // chained
    println!("{}", -1i8 as u8 as i32);
    println!("{}", -1i8 as i32 as u32);
    println!("{}", 200u8 as i8 as i64);
    println!("{}", 3.7f64 as i32 as f64);
}
