fn main() {
    let pairs: [(i32, i32); 8] = [(7, 3), (-7, 3), (7, -3), (-7, -3), (0, 5), (1, -1), (-9, 2), (9, -2)];
    for (a, b) in pairs.iter() {
        println!("{} {}: / {} % {} div_euclid {} rem_euclid {}", a, b, a / b, a % b, a.div_euclid(*b), a.rem_euclid(*b));
    }
    println!("{} {}", 7u32 / 2, 7u32 % 2);
    println!("{} {}", (-7i64).rem_euclid(3), (-7i64).div_euclid(-3));
    println!("{} {}", (-7i8) % 3, (-128i8) / 127);
    println!("{:?} {:?}", 5i32.checked_div(0), i32::MIN.checked_div(-1));
    println!("{:?} {:?}", 5i32.checked_rem(0), i32::MIN.checked_rem(-1));
    println!("{} {}", i32::MIN.wrapping_div(-1), i32::MIN.wrapping_rem(-1));
    println!("{:?}", i32::MIN.overflowing_div(-1));
    println!("{:?}", 7i32.checked_rem_euclid(0));
    println!("{:?}", (-7i32).checked_div_euclid(2));
    println!("{}", (-7.5f64).rem_euclid(2.0));
    println!("{}", (-7.5f64).div_euclid(2.0));
    println!("{}", -7.5f64 % 2.0);
    println!("{}", 7.5f64 % -2.0);
    println!("{}", (-1i32).rem_euclid(26));
    // modular arithmetic for index wrapping
    let len = 5i32;
    for step in [-12i32, -5, -1, 0, 4, 11] {
        print!("{} ", step.rem_euclid(len));
    }
    println!();
    println!("{}", 17u64.div_ceil(5));
    println!("{}", 100u32.abs_diff(250));
    println!("{}", (-100i32).abs_diff(250));
    println!("{}", i8::MIN.unsigned_abs());
    println!("{}", 10i32.midpoint(-3));
    let x = 0i32;
    let r = 10 / (x + 2);
    println!("{}", r);
    let zero = "0".parse::<i32>().unwrap();
    println!("about to divide");
    println!("{}", 10 / zero);
}
