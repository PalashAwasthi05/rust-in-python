fn main() {
    let a: f32 = 0.1;
    let b: f32 = 0.2;
    println!("{} {:?}", a, a);
    println!("{} {:?}", a + b, a + b);
    println!("{}", 1.0f32 / 3.0);
    println!("{:?}", 16777216.0f32);
    println!("{:?}", 16777217.0f32);
    println!("{} {:?}", f32::MAX, f32::MAX);
    println!("{} {:?}", f32::MIN_POSITIVE, f32::MIN_POSITIVE);
    println!("{:?}", f32::EPSILON);
    println!("{}", f32::NAN);
    println!("{:?}", -f32::INFINITY);
    println!("{:?}", 1e10f32);
    println!("{:?}", 1e16f32);
    println!("{:?}", 1e-7f32);
    println!("{:.3}", 2.0f32 / 3.0);
    println!("{:e}", 1234.5f32);
    let v: Vec<f32> = vec![1.0, 2.5, 0.1, 3.14159];
    println!("{:?}", v);
    let t = (1.0f32, 2.0f64);
    println!("{:?}", t);
    let c = 0.1f32 as f64;
    println!("{}", c);
    println!("{}", (0.7f32 * 3.0) as f64);
    println!("{}", 0.7f32 * 3.0);
    let mut acc = 0.0f32;
    for _ in 0..10 {
        acc += 0.1;
    }
    println!("{} {:?}", acc, acc);
    println!("{:?}", 1.0e-40f32);
    println!("{}", 33554432.0f32 + 1.0);
}
