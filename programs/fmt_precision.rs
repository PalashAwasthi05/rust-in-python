fn main() {
    let values = [0.5, 1.5, 2.5, 3.5, -0.5, -2.5, 0.125, 0.375, 2.675, 1.005, 0.045, 1e-10, 123.456];
    for v in values.iter() {
        println!("{:.0} {:.1} {:.2} {:.3}", v, v, v, v);
    }
    println!("{:.5}", 1.0f64 / 3.0);
    println!("{:.20}", 0.1);
    println!("{:.30}", 0.1f32);
    println!("{:.3}", f64::NAN);
    println!("{:.2}", f64::INFINITY);
    println!("{:8.2}|", -f64::INFINITY);
    println!("{:.1}", -0.04);
    println!("{:.0}", -0.4);
    println!("{:.2}", 1e21);
    println!("{:.3?}", 1.0);
    println!("{:.1?}", vec![1.25, 2.35]);
    println!("{:.2}", 9.995);
    println!("{:.2}", 9.999);
    let p = 4;
    println!("{:.*}", 2, 3.14159);
    println!("{:.p$}", 3.14159);
    println!("{:.1$}", 2.71828, p);
    println!("{:10.3}|{:<10.3}|{:^10.3}|", 3.14159, 3.14159, 3.14159);
    println!("{:.3}", "abcdef");
    println!("{:5.2}|", "abcdef");
    println!("{:.0}", 0.5f32);
    println!("{:.1}", 0.25f32);
    println!("{:.1}", 0.35f32);
    println!("{:.2}", 5);
    println!("{:.3}", -0.0);
    println!("{:.1}", 0.05);
}
