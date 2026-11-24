fn describe<T: std::fmt::Display + Copy>(name: &str, min: T, max: T) {
    println!("{}: {} ..= {}", name, min, max);
}

fn main() {
    describe("i8", i8::MIN, i8::MAX);
    describe("i16", i16::MIN, i16::MAX);
    describe("i32", i32::MIN, i32::MAX);
    describe("i64", i64::MIN, i64::MAX);
    describe("i128", i128::MIN, i128::MAX);
    describe("isize", isize::MIN, isize::MAX);
    describe("u8", u8::MIN, u8::MAX);
    describe("u16", u16::MIN, u16::MAX);
    describe("u32", u32::MIN, u32::MAX);
    describe("u64", u64::MIN, u64::MAX);
    describe("u128", u128::MIN, u128::MAX);
    describe("usize", usize::MIN, usize::MAX);
    println!("{}", std::mem::size_of::<u16>() + std::mem::size_of::<i128>());
    let big: u64 = u64::MAX;
    println!("{}", big / 3);
    println!("{}", big % 1_000_000_007);
    println!("{}", big - 1);
    println!("{}", (big >> 1) + 1);
    let half = u64::MAX / 2 + 1;
    println!("{}", half);
    let prod = (u32::MAX as u64) * (u32::MAX as u64);
    println!("{}", prod);
    let wide = (u64::MAX as u128) * (u64::MAX as u128);
    println!("{}", wide);
    let mulhi = ((0xDEADBEEFCAFEBABEu64 as u128 * 0x123456789ABCDEFu64 as u128) >> 64) as u64;
    println!("{}", mulhi);
    let modmul = (123456789123u64 as u128 * 987654321987u64 as u128 % 1_000_000_007u128) as u64;
    println!("{}", modmul);
    println!("{}", i128::MAX / -7);
    println!("{}", i128::MIN % 1000);
    println!("{}", u128::MAX >> 64);
    let mut fib = [0u128; 150];
    fib[1] = 1;
    for i in 2..150 { fib[i] = fib[i - 1] + fib[i - 2]; }
    println!("{}", fib[149]);
    let x: i16 = -300;
    let y: i16 = 200;
    println!("{}", (x as i32 * y as i32) / 1000);
    let a: u16 = 60000;
    let b: u16 = 5000;
    println!("{}", a as u32 + b as u32);
    println!("{:?}", a.checked_add(b));
    let mut c: i8 = 0;
    for _ in 0..127 { c += 1; }
    println!("{}", c);
    println!("{}", 1u32 == 1);
    println!("{}", -1i32 < 0);
    println!("{:?}", (5i32).cmp(&3));
    println!("{}", u8::try_from(300i32).is_err());
    println!("{:?}", u8::try_from(255i32));
    println!("{:?}", i8::try_from(-129i64).map_err(|e| e.to_string()));
    let t: u32 = 70000u64.try_into().unwrap();
    println!("{}", t);
    let w: i64 = i64::from(-5i32);
    println!("{}", w);
    let u: u64 = u64::from(7u8);
    println!("{}", u);
    let f: f64 = f64::from(3u32);
    println!("{}", f);
    let g: f64 = 2.5f32.into();
    println!("{}", g);
    let ch: u32 = 'x'.into();
    println!("{}", ch);
    let small: usize = usize::try_from(-1i32).unwrap_or(99);
    println!("{}", small);
    println!("{}", i32::MAX.count_ones() + u8::MAX.count_ones());
    println!("{}", (i32::MAX as i64 + 1) as i32);
    println!("{:?}", 5u32.checked_next_power_of_two());
    println!("{}", 255u8.to_string().len());
    println!("{:>5}|{:<5}|{:^5}|{:05}|{:+}", 42, 42, 42, -42, 42);
}
