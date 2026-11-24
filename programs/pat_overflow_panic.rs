// Checked/wrapping arithmetic, then an overflow panic (exit 101).
fn factorial(n: u8) -> u8 {
    let mut acc: u8 = 1;
    for i in 1..=n {
        acc *= i;
    }
    acc
}

fn main() {
    let a: u8 = 250;
    println!("checked: {:?} {:?}", a.checked_add(5), a.checked_add(6));
    println!("wrapping: {}", a.wrapping_add(10));
    println!("saturating: {}", a.saturating_add(10));
    let (v, o) = a.overflowing_add(10);
    println!("overflowing: {} {}", v, o);
    let x: i32 = i32::MAX;
    println!("{:?}", x.checked_mul(2));
    println!("{}", (-128i8).wrapping_neg());
    println!("{:?}", 5u32.checked_sub(6));
    for n in 1..=6 {
        println!("{}! = {}", n, factorial(n));
    }
}
