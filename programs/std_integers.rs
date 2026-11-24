// Integer methods and constants.
fn collatz_len(mut n: u64) -> u32 {
    let mut steps = 0;
    while n != 1 {
        n = if n % 2 == 0 { n / 2 } else { 3 * n + 1 };
        steps += 1;
    }
    steps
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

fn main() {
    println!("pow {} {} {}", 2i32.pow(10), 3u64.pow(20), (-2i64).pow(3));
    println!("checked_add {:?} {:?}", 250u8.checked_add(5), 250u8.checked_add(6));
    println!("checked_sub {:?} {:?}", 3u32.checked_sub(4), 4u32.checked_sub(3));
    println!("checked_mul {:?} {:?}", i32::MAX.checked_mul(2), 1000i32.checked_mul(1000));
    println!("checked_div {:?} {:?}", 10i32.checked_div(0), i32::MIN.checked_div(-1));
    println!("checked_pow {:?} {:?}", 2u32.checked_pow(31), 2u32.checked_pow(32));
    println!("checked_rem {:?} checked_neg {:?}", 7i32.checked_rem(0), i8::MIN.checked_neg());
    println!("saturating {} {} {} {}", 250u8.saturating_add(10), 5u8.saturating_sub(10), i32::MIN.saturating_sub(1), 100i8.saturating_mul(2));
    println!("wrapping {} {} {} {}", 250u8.wrapping_add(10), 0u8.wrapping_sub(1), i32::MAX.wrapping_add(1), 200u8.wrapping_mul(2));
    println!("wrapping_neg {} wrapping_shl {}", 1u8.wrapping_neg(), 1u32.wrapping_shl(33));
    println!("overflowing {:?} {:?}", 255u8.overflowing_add(1), 5i32.overflowing_sub(3));
    println!("abs_diff {} {}", 3u32.abs_diff(10), (-5i32).abs_diff(7));
    println!("rem_euclid {} {} div_euclid {} {}", (-7i32).rem_euclid(3), 7i32.rem_euclid(-3), (-7i32).div_euclid(3), 7i32.div_euclid(-3));
    println!("% {} {}", -7 % 3, 7 % -3);
    println!("abs {} signum {} {} {}", (-42i32).abs(), (-42i32).signum(), 0i32.signum(), 9i64.signum());
    println!("is_positive {} is_negative {}", 3i32.is_positive(), (-3i32).is_negative());
    println!("leading_zeros {} trailing_zeros {} count_ones {} count_zeros {}", 1u32.leading_zeros(), 8u16.trailing_zeros(), 255u8.count_ones(), 0b1010u8.count_zeros());
    println!("leading_ones {} trailing_ones {}", 0xF0u8.leading_ones(), 0b0111u8.trailing_ones());
    println!("is_power_of_two {} {} next_power_of_two {}", 64u32.is_power_of_two(), 65u32.is_power_of_two(), 65u32.next_power_of_two());
    println!("rotate {} {} swap_bytes {:#x}", 0b1000_0001u8.rotate_left(1), 1u8.rotate_right(1), 0x1234u16.swap_bytes());
    println!("reverse_bits {:#010b}", 0b0000_0110u8.reverse_bits());
    println!("min max {} {} {} {}", i32::MAX, i32::MIN, u8::MAX, i64::MIN);
    println!("u128 max {} bits {}", u128::MAX, u64::BITS);
    println!("min/max methods {} {} clamp {}", 3.min(9), 3.max(9), 15.clamp(0, 10));
    println!("isqrt-ish {}", (1..100u32).take_while(|x| x * x <= 50).last().unwrap());
    println!("ilog {} {} {}", 1000u32.ilog10(), 1024u32.ilog2(), 81u32.ilog(3));
    println!("gcd {}", gcd(462, 1071));
    let longest = (1..30u64).max_by_key(|&n| collatz_len(n)).unwrap();
    println!("collatz longest under 30: {} ({} steps)", longest, collatz_len(longest));
    println!("casts {} {} {} {} {}", 300i32 as u8, -1i32 as u32, 3.99f64 as i32, -3.99f64 as i32, 1e20 as i32);
    println!("casts {} {} {} {}", -1i8 as u8, 255u8 as i8, u64::MAX as i64, (65u8 + 1) as char);
    println!("float casts {} {} {}", f64::NAN as i32, -1.5 as u8, 256.7 as u8);
    println!("from {} {} {}", i64::from(7i32), u32::from(true), f64::from(3i32));
    println!("try_from {:?} {:?}", u8::try_from(300i32), i8::try_from(-5i64));
    let r: Result<u16, _> = 70000u32.try_into();
    println!("try_into {:?}", r.is_err());
    println!("to_string {} {}", 255.to_string() + "!", (-0i32).to_string());
    println!("bit ops {} {} {} {} {}", 0b1100 & 0b1010, 0b1100 | 0b1010, 0b1100 ^ 0b1010, !0u8, 1u64 << 40);
    println!("shr signed {} unsigned {}", -16i32 >> 2, 0xF0u8 >> 4);
    println!("to_be_bytes {:?} from_le {}", 258u16.to_be_bytes(), u32::from_le_bytes([1, 0, 0, 0]));
    println!("midpoint-ish {}", (i32::MAX / 2 + i32::MAX / 2));
    let digits: u32 = 98765u32.to_string().chars().map(|c| c.to_digit(10).unwrap()).sum();
    println!("digit sum {}", digits);
    println!("sum of squares {}", (1..=100u64).map(|x| x * x).sum::<u64>());
    println!("checked chain {:?}", 10u8.checked_mul(20).and_then(|x| x.checked_add(60)));
    println!("usize max eq u64 max {}", usize::MAX as u64 == u64::MAX);
    println!("i16 range {}..={}", i16::MIN, i16::MAX);
    println!("pow overflow-safe {:?}", 10u64.checked_pow(19));
    println!("saturating_pow {}", 3i8.saturating_pow(5));
    println!("unsigned_abs {} {}", (-128i8).unsigned_abs(), i32::MIN.unsigned_abs());
}
