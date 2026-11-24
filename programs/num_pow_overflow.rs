fn main() {
    let mut base: u64 = 3;
    for e in [10u32, 20, 40] {
        println!("3^{} = {}", e, base.pow(e));
    }
    base = 10;
    println!("{:?}", base.checked_pow(20));
    println!("{}", base.saturating_pow(20));
    println!("{}", base.wrapping_pow(20));
    println!("{:?}", base.overflowing_pow(20));
    println!("{}", base.pow(20));
}
