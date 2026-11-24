fn factorial(n: u32) -> u32 {
    (1..=n).product()
}

fn main() {
    for n in [5, 10, 12] {
        println!("{}! = {}", n, factorial(n));
    }
    println!("{:?}", 255u8.checked_add(1));
    println!("{}", 255u8.wrapping_add(1));
    println!("{:?}", 200u8.overflowing_add(100));
    println!("{}", 100u8.saturating_add(200));
    let mut x: u8 = 250;
    for _ in 0..10 {
        x += 1;
        println!("x = {}", x);
    }
}
