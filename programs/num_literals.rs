fn takes_u8(x: u8) -> u8 { x }
fn takes_i64(x: i64) -> i64 { x * 2 }
fn generic_double<T: std::ops::Add<Output = T> + Copy>(x: T) -> T { x + x }
fn largest<T: PartialOrd + Copy>(xs: &[T]) -> T {
    let mut m = xs[0];
    for &x in xs { if x > m { m = x; } }
    m
}

fn main() {
    println!("{} {} {} {}", 1_000_000, 0xff, 0o17, 0b1010);
    println!("{} {} {}", 1e3, 2.5e-3, 1E2);
    println!("{} {}", 10u8, 3.0f32);
    println!("{} {}", 0xFFu8, 0b1111_0000u8);
    println!("{}", 1_2_3);
    println!("{}", 0x_dead_beef_u32);
    println!("{}", 1_000i64 * 1_000_000_000);
    println!("{}", 100_000_000_000u64);
    println!("{}", 0.000_001f64);
    println!("{}", 5f64);
    println!("{}", 12E-1);
    println!("{}", takes_u8(255));
    println!("{}", takes_i64(5_000_000_000));
    println!("{}", generic_double(21));
    println!("{}", generic_double(1.25));
    println!("{}", generic_double(200u8 / 2));
    println!("{}", largest(&[3, 9, 2]));
    println!("{}", largest(&[0.5, -1.0]));
    let x = 3_000_000_000u32;
    println!("{}", x);
    let y: u64 = 1 << 40;
    println!("{}", y);
    let z = 1u64 << 40;
    println!("{}", z);
    let w: i128 = 1 << 100;
    println!("{}", w);
    let v = vec![1u16, 2, 3];
    let total: u16 = v.iter().sum();
    println!("{}", total);
    let mut counter = 0;
    counter += 1u8;
    println!("{}", counter);
    let big = 4_294_967_296;
    let r: i64 = big;
    println!("{}", r);
    let h = 0xFFFF_FFFF_FFFF_FFFFu64;
    println!("{}", h);
    let neg = -0x80i32;
    println!("{}", neg);
    println!("{}", i8::MIN);
    println!("{}", -128i8);
    println!("{}", b'A');
    println!("{}", b'\n');
    println!("{}", b'\x7f');
    println!("{:?}", b"hi");
    println!("{}", '\u{41}');
    println!("{}", std::u32::MAX);
    println!("{}", core::i16::MAX);
    let arr = [0u8; 4];
    println!("{:?}", arr);
    let f = 2.0_f64;
    println!("{}", f);
    let e = 1e-3_f32;
    println!("{}", e);
    let t = (1u8, 2i16, 3u32, 4i64, 5usize, 6.0f32);
    println!("{:?}", t);
    let max_i = i32::MAX;
    let as_f = max_i as f32;
    println!("{}", as_f);
    let lit = 2147483647;
    println!("{}", lit);
    println!("{}", -2147483648i64 - 1);
}
