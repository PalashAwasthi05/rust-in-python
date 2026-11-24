fn main() {
    // wrapping
    println!("{} {} {}", 250u8.wrapping_add(10), 5u8.wrapping_sub(10), 16u8.wrapping_mul(17));
    println!("{} {}", i8::MAX.wrapping_add(1), i8::MIN.wrapping_sub(1));
    println!("{} {}", i64::MAX.wrapping_mul(3), u32::MAX.wrapping_add(u32::MAX));
    println!("{} {}", 3u8.wrapping_pow(7), (-3i16).wrapping_pow(11));
    println!("{}", 0u32.wrapping_sub(1));
    println!("{}", u128::MAX.wrapping_mul(u128::MAX));
    println!("{}", i128::MIN.wrapping_neg());
    // checked
    println!("{:?} {:?}", 200u8.checked_add(55), 200u8.checked_add(56));
    println!("{:?} {:?}", 100i8.checked_mul(2), (-64i8).checked_mul(2));
    println!("{:?} {:?}", 2u32.checked_pow(31), 2u32.checked_pow(32));
    println!("{:?} {:?}", 10u64.checked_pow(19), 10u64.checked_pow(20));
    println!("{:?}", (-2i32).checked_pow(31));
    println!("{:?}", u64::MAX.checked_add(1).unwrap_or(0));
    // saturating
    println!("{} {}", 250u8.saturating_add(10), 5u8.saturating_sub(10));
    println!("{} {}", 100i8.saturating_add(100), (-100i8).saturating_sub(100));
    println!("{} {}", i32::MAX.saturating_mul(2), i32::MIN.saturating_mul(2));
    println!("{} {}", (-5i32).saturating_mul(i32::MAX), 3u16.saturating_pow(20));
    println!("{}", i8::MIN.saturating_neg());
    println!("{}", i8::MIN.saturating_abs());
    // overflowing
    println!("{:?} {:?}", 250u8.overflowing_add(10), 5u8.overflowing_sub(10));
    println!("{:?}", i32::MAX.overflowing_mul(2));
    println!("{:?}", 3u8.overflowing_pow(6));
    println!("{:?}", i64::MIN.overflowing_sub(1));
    // pow
    println!("{} {} {}", 2i32.pow(10), (-3i64).pow(3), 10u128.pow(30));
    println!("{}", 7u64.pow(0));
    // min max clamp signum
    println!("{} {} {}", 5i32.min(3), 5i32.max(9), 15i32.clamp(0, 10));
    println!("{} {} {}", (-5i32).signum(), 0i32.signum(), 7i64.signum());
    println!("{} {}", (-5i32).is_negative(), 5i32.is_positive());
    println!("{} {} {}", i16::MAX, i16::MIN, u16::MAX);
    println!("{} {}", i128::MAX, u128::MAX);
    println!("{} {}", u8::BITS, i64::BITS);
    // isqrt / ilog
    println!("{} {} {}", 99u32.isqrt(), 100u64.isqrt(), u64::MAX.isqrt());
    println!("{} {} {}", 1u32.ilog2(), 1024u32.ilog2(), u64::MAX.ilog2());
    println!("{} {} {}", 9u32.ilog10(), 1000u32.ilog10(), 999_999u64.ilog10());
    println!("{}", 81u32.ilog(3));
    println!("{:?} {:?}", 0u32.checked_ilog2(), 8u32.checked_ilog2());
    // sum/product
    let v: Vec<u32> = vec![1, 2, 3, 4, 5];
    let s: u32 = v.iter().sum();
    let p: u64 = v.iter().map(|&x| x as u64).product();
    println!("{} {}", s, p);
    let total: i64 = (1..=100).sum();
    println!("{}", total);
    let mx = v.iter().copied().max().unwrap();
    println!("{}", mx);
}
