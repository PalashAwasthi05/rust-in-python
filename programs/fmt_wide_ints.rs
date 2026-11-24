fn main() {
    println!("{}", i128::MAX);
    println!("{}", i128::MIN);
    println!("{}", u128::MAX);
    println!("{:?}", i128::MIN);
    let big: u128 = 1 << 100;
    println!("{} {:x} {:#b}", big, big, 1u128 << 3);
    println!("{:>45}|", u128::MAX);
    println!("{:+}", i128::MAX);
    println!("{:050}", i128::MIN);
    println!("{} {}", i64::MIN, i64::MAX);
    println!("{} {}", u64::MAX, i8::MIN);
    println!("{:o}", u128::MAX);
    println!("{:X}", i128::MIN);
    println!("{}", -170141183460469231731687303715884105727i128);
    let v: Vec<i128> = vec![-1, 0, 1];
    println!("{:?}", v);
    println!("{:e}", u128::MAX);
    println!("{}", usize::MAX);
    let product: u128 = 18446744073709551615u128 * 18446744073709551615u128;
    println!("{}", product);
    println!("{:?}", (u128::MAX, i128::MIN));
}
