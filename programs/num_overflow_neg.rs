fn negate(x: i32) -> i32 { -x }
fn main() {
    println!("{}", negate(5));
    println!("{}", (-7i32).abs());
    println!("{:?} {:?}", i32::MIN.checked_abs(), i32::MIN.checked_neg());
    println!("{} {}", i32::MIN.wrapping_abs(), i32::MIN.wrapping_neg());
    println!("{:?}", i32::MIN.overflowing_neg());
    println!("{}", i64::MIN.unsigned_abs());
    let m = i32::MIN;
    println!("{}", m.abs());
}
