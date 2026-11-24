fn main() {
    let x: u64 = (1 << 60) + (1 << 36) + 1;
    println!("{}", x as f32);
    println!("{}", (x - 1) as f32);
    println!("{}", (x + (1 << 37)) as f32);
    let y: i64 = -((1 << 40) + (1 << 16) + 1);
    println!("{}", y as f32);
    println!("{}", u128::MAX as f32);
    println!("{}", (u128::MAX >> 1) as f32);
    println!("{}", 16777217u32 as f32);
    println!("{}", 16777219u32 as f32);
    println!("{}", -16777219i32 as f32);
    println!("{}", u64::MAX as f32);
    println!("{}", i64::MIN as f32);
    println!("{}", 33554435u64 as f32);
}
