fn checked_average(xs: &[u32]) -> Option<u32> {
    let mut total: u32 = 0;
    for &x in xs {
        total = total.checked_add(x)?;
    }
    total.checked_div(xs.len() as u32)
}

fn hash(s: &str) -> u32 {
    // a tiny FNV-style hash that relies on wrapping arithmetic
    let mut h: u32 = 2166136261;
    for b in s.bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(16777619);
    }
    h
}

fn main() {
    let x: u8 = 250;
    println!("u8 range {}..={}, i8 range {}..={}", u8::MIN, u8::MAX, i8::MIN, i8::MAX);
    println!("checked_add(5) = {:?}, checked_add(6) = {:?}", x.checked_add(5), x.checked_add(6));
    println!("wrapping_add(10) = {}", x.wrapping_add(10));
    println!("saturating_add(10) = {}", x.saturating_add(10));
    println!("overflowing_add(10) = {:?}", x.overflowing_add(10));
    println!("checked_sub from 0u8: {:?}", 0u8.checked_sub(1));
    println!("saturating_sub from 0u8: {}", 0u8.saturating_sub(1));
    println!("wrapping_sub from 0u8: {}", 0u8.wrapping_sub(1));

    let m = i32::MAX;
    println!("i32::MAX = {}", m);
    println!("wrapping_add 1 = {}", m.wrapping_add(1));
    println!("checked_mul 2 = {:?}", m.checked_mul(2));
    println!("saturating_mul 2 = {}", m.saturating_mul(2));
    println!("overflowing_mul 2 = {:?}", m.overflowing_mul(2));
    println!("i32::MIN.checked_abs() = {:?}", i32::MIN.checked_abs());
    println!("i32::MIN.wrapping_abs() = {}", i32::MIN.wrapping_abs());
    println!("i32::MIN.checked_neg() = {:?}", i32::MIN.checked_neg());
    println!("i32::MIN / -1 checked = {:?}", i32::MIN.checked_div(-1));
    println!("10 / 0 checked = {:?}", 10i32.checked_div(0));
    println!("checked_rem 0 = {:?}", 10u32.checked_rem(0));
    println!("pow: {:?} {:?} {}", 2u32.checked_pow(31), 2u32.checked_pow(32), 3u8.wrapping_pow(6));
    println!("shifts: {:?} {:?} {}", 1u32.checked_shl(31), 1u32.checked_shl(32), 1u32.wrapping_shl(33));

    let readings = [4_000_000_000u32, 200_000_000, 94_967_295];
    println!("average safe? {:?}", checked_average(&readings));
    println!("average ok: {:?}", checked_average(&[10, 20, 31]));
    println!("average empty: {:?}", checked_average(&[]));

    for word in ["", "a", "rust", "Rust"] {
        println!("hash({:?}) = {:#010x}", word, hash(word));
    }

    let mut counter: u8 = 253;
    for _ in 0..5 {
        counter = counter.wrapping_add(1);
        print!("{} ", counter);
    }
    println!();
    let wrapped = std::num::Wrapping(250u8) + std::num::Wrapping(10u8);
    println!("Wrapping type: {}", wrapped);
    let sum: u64 = (1..=100u64).map(|n| n.pow(5)).sum();
    println!("sum of fifth powers to 100 = {}", sum);
    println!("float overflow: {} {} {}", f64::MAX * 2.0, -f64::MAX * 2.0, (0.0f64 / 0.0).is_nan());
}
