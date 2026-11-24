// Parsing numbers, chars and bools, with errors.
use std::num::ParseIntError;

fn sum_csv(line: &str) -> Result<i64, ParseIntError> {
    let mut total = 0;
    for field in line.split(',') {
        total += field.trim().parse::<i64>()?;
    }
    Ok(total)
}

fn main() {
    println!("{:?}", "42".parse::<i32>());
    println!("{:?}", "-17".parse::<i64>());
    println!("{:?}", "+8".parse::<u8>());
    println!("{:?}", "256".parse::<u8>());
    println!("{:?}", "-1".parse::<u32>());
    println!("{:?}", "".parse::<i32>());
    println!("{:?}", "12a".parse::<i32>());
    println!("{:?}", "-129".parse::<i8>());
    println!("{:?}", "18446744073709551615".parse::<u64>());
    println!("{:?}", "340282366920938463463374607431768211455".parse::<u128>());
    println!("{:?}", "3.25".parse::<f64>());
    println!("{:?}", "1e3".parse::<f64>());
    println!("{:?}", "-0.5".parse::<f32>());
    println!("{:?}", "inf".parse::<f64>());
    println!("{:?}", "abc".parse::<f64>());
    println!("{:?}", "x".parse::<char>());
    println!("{:?}", "xy".parse::<char>());
    println!("{:?}", "true".parse::<bool>());
    println!("{:?}", "yes".parse::<bool>());
    println!("{:?}", "99".parse::<usize>());
    match "999999".parse::<i16>() {
        Ok(n) => println!("ok {}", n),
        Err(e) => println!("error: {}", e),
    }
    match "".parse::<u32>() {
        Ok(n) => println!("ok {}", n),
        Err(e) => println!("error: {}", e),
    }
    match "4x".parse::<u32>() {
        Ok(n) => println!("ok {}", n),
        Err(e) => println!("error: {}", e),
    }
    match "nope".parse::<f64>() {
        Ok(n) => println!("ok {}", n),
        Err(e) => println!("error: {}", e),
    }
    match "maybe".parse::<bool>() {
        Ok(n) => println!("ok {}", n),
        Err(e) => println!("error: {}", e),
    }
    match "ab".parse::<char>() {
        Ok(n) => println!("ok {}", n),
        Err(e) => println!("error: {}", e),
    }
    println!("{:?}", sum_csv("1, 2, 3, 40"));
    println!("{:?}", sum_csv("1, two, 3"));
    println!("{:?}", i64::from_str_radix("-ff", 16));
    println!("{:?}", u8::from_str_radix("101", 2));
    println!("{:?}", u32::from_str_radix("zz", 36));
    println!("{:?}", i32::from_str_radix("12", 3));
    println!("{:?}", u8::from_str_radix("1012", 2));
    let nums: Result<Vec<i32>, _> = "1 2 3".split(' ').map(str::parse::<i32>).collect();
    println!("{:?}", nums);
    let bad: Result<Vec<i32>, _> = "1 x 3".split(' ').map(|s| s.parse::<i32>()).collect();
    println!("{:?}", bad);
    let good_only: Vec<u16> = "5 -3 70000 12 abc 9".split_whitespace().filter_map(|s| s.parse().ok()).collect();
    println!("{:?}", good_only);
    let n: i32 = "  77 ".trim().parse().unwrap_or(0);
    let m: i32 = "oops".parse().unwrap_or(-1);
    println!("{} {}", n, m);
    let value: f64 = "2.5".parse().expect("float");
    println!("{}", value * 2.0);
    let e = "q".parse::<i32>().unwrap_err();
    println!("kind: {:?}", e.kind());
    let big = "300".parse::<u8>().unwrap_err();
    println!("kind: {:?} / {}", big.kind(), big);
}
