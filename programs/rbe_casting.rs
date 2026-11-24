#![allow(overflowing_literals)]

fn main() {
    let decimal = 65.4321_f32;
    let integer = decimal as u8;
    let character = integer as char;
    println!("casting: {} -> {} -> {}", decimal, integer, character);

    // integer to integer: truncation and sign reinterpretation
    println!("1000 as u16 is: {}", 1000 as u16);
    println!("1000 as u8 is : {}", 1000 as u8);
    println!("  -1 as u8 is : {}", (-1i8) as u8);
    println!("1000 mod 256 is : {}", 1000 % 256);
    println!(" 128 as i16 is: {}", 128 as i16);
    println!(" 128 as i8 is : {}", 128 as i8);
    println!("1000 as u8 is : {}", 1000 as u8);
    println!(" 232 as i8 is : {}", 232 as i8);
    println!("-1i32 as u32 = {}, -1i64 as u64 = {}", -1i32 as u32, -1i64 as u64);
    println!("u32::MAX as i32 = {}, i64::MIN as i32 = {}", u32::MAX as i32, i64::MIN as i32);
    println!("-5i8 as i64 = {} (sign extended), 251u8 as i64 = {}", -5i8 as i64, 251u8 as i64);
    println!("0xABCD_1234u32 as u16 = {:#x}", 0xABCD_1234u32 as u16);

    // float to int saturates and NaN becomes 0
    println!(" 300.0 as u8 is : {}", 300.0_f32 as u8);
    println!("-100.0 as u8 is : {}", -100.0_f32 as u8);
    println!("   nan as u8 is : {}", f32::NAN as u8);
    println!("  9.99 as i32 is: {}", 9.99f64 as i32);
    println!(" -9.99 as i32 is: {}", -9.99f64 as i32);
    println!("1e20 as i64 = {}, -1e20 as i64 = {}", 1e20f64 as i64, -1e20f64 as i64);
    println!("inf as u16 = {}, -inf as i16 = {}", f64::INFINITY as u16, f64::NEG_INFINITY as i16);

    // int to float and float to float
    println!("u64::MAX as f32 = {}", u64::MAX as f32);
    println!("16777217 as f32 = {} (precision loss)", 16_777_217i32 as f32);
    println!("0.1f32 as f64 = {}", 0.1f32 as f64);
    println!("1e40f64 as f32 = {}", 1e40f64 as f32);
    println!("i8::MIN as f64 = {}", i8::MIN as f64);

    // bool and char casts
    println!("true as i32 = {}, false as u8 = {}", true as i32, false as u8);
    println!("'A' as u32 = {}, 'é' as u8 = {}, '🦀' as u32 = {:#x}", 'A' as u32, 'é' as u8, '🦀' as u32);
    println!("97u8 as char = {:?}", 97u8 as char);
    println!("char::from_u32(0x1F980) = {:?}, char::from_u32(0xD800) = {:?}", char::from_u32(0x1F980), char::from_u32(0xD800));
    println!("char::from_digit(7, 10) = {:?}, 'f'.to_digit(16) = {:?}", char::from_digit(7, 10), 'f'.to_digit(16));

    // checked conversions as the safe alternative
    let big: i64 = 300;
    println!("u8::try_from(300) = {:?}", u8::try_from(big).is_err());
    println!("i8::try_from(-128i32) = {:?}", i8::try_from(-128i32));
    println!("usize from u16: {}", usize::from(65535u16));
    let mixed = 3u8 as f64 * 2.5 + -2i16 as f64;
    println!("mixed arithmetic needs casts: {}", mixed);
    let index: usize = 7;
    let offset: i32 = -3;
    let target = (index as i32 + offset) as usize;
    println!("signed offset into index: {}", target);
}
