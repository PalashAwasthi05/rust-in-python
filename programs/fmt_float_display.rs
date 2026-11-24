fn main() {
    let values: [f64; 18] = [
        0.1 + 0.2, 1e21, 1e-7, 1e16, 1e15, 123456789.0, 0.3, -0.0, 0.0, 1.0, -1.5,
        f64::MAX, f64::MIN_POSITIVE, f64::EPSILON, 2.5e-308, 1.0 / 3.0, 100.0, 1e300,
    ];
    for v in values.iter() {
        println!("{} | {:?}", v, v);
    }
    let nan = f64::NAN;
    let inf = f64::INFINITY;
    println!("{} {:?} {} {:?}", nan, nan, inf, -inf);
    println!("{} {:?}", f64::MIN, f64::MIN);
    let x = 7.0_f64;
    println!("{} {:?} {x} {x:?}", x, x);
    let small = 0.000123;
    println!("{} {:?}", small, small);
    let tiny = 1.5e-5;
    println!("{} {:?}", tiny, tiny);
    println!("{:?} {:?}", 1e-5, 0.0001);
    println!("{:?} {:?} {:?}", 1e15, 1e16, 1.5e16);
    println!("{} {}", 1e22, 12345678901234567890.0);
    println!("{}", 2.0f64.powi(60));
    println!("{}", 2.0f64.sqrt());
    println!("{}", std::f64::consts::PI);
    println!("{:?}", std::f64::consts::E);
    let total: f64 = [0.1; 10].iter().sum();
    println!("{} {:?}", total, total);
    println!("{:?}", -0.0 * 1.0);
    println!("{}", 5e-324);
    println!("{:?}", 5e-324);
}
