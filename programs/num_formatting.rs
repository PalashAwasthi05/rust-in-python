fn rot13(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'a'..='z' => ((c as u8 - b'a' + 13) % 26 + b'a') as char,
            'A'..='Z' => ((c as u8 - b'A' + 13) % 26 + b'A') as char,
            _ => c,
        })
        .collect()
}

fn main() {
    println!("{}", 1e300f64);
    println!("{}", 1.5e-300f64);
    println!("{:?}", 0.1 + 0.2);
    println!("{}", f64::MAX + f64::MAX);
    println!("{}", 1.0f64 / 0.0);
    println!("{}", 0.0f64 / 0.0);
    println!("{}", -f64::NAN);
    println!("{:?}", -1.0f64 / 0.0);
    println!("{:e}", 1500);
    println!("{:e}", 0.0f64);
    println!("{:>12.3e}|", 98765.4321f64);
    println!("{:+.2} {:+.2}", 3.14159, -0.001);
    println!("{:08.3} {:08.3}", -2.5, 2.5);
    println!("{:x} {:X}", -1i128, 255u8);
    println!("{:#b} {:#o} {:#X}", 5u8, 8u16, 3054);
    println!("{:>+6}|{:<+6}|{:^+7}", 5, -5, 5);
    println!("{:#06x} {:#06b}", 255, 5);
    println!("{:10.1}|{:<10.1}|{:^10.1}|", 1.25, 1.35, -1.45);
    println!("{:.0} {:.0} {:.0} {:.0}", 0.5, 1.5, 2.5, -0.5);
    println!("{:.1} {:.1} {:.1}", 0.05, 0.15, 0.25);
    println!("{:.3}", 1.0005);
    println!("{:.20}", 0.1);
    println!("{:.15}", 1.0f32 / 3.0);
    println!("{}", 100000000000000000000.0f64);
    println!("{}", 123456789.123456789f64);
    println!("{}", 0.000001234f64);
    println!("{:?}", 0.000001234f64);
    println!("{:?}", 1234567890123456789.0f64);
    println!("{:?}", 0.1f32 as f64);
    println!("{:5}|{:<5}|{:>5}|{:^5}|", 1.5, 1.5, 1.5, 1.5);
    println!("{:05}", -1.5);
    println!("{:width$.prec$}", 3.14159265, width = 10, prec = 3);
    let w = 6;
    println!("{:>w$}|{:0>w$}", 42, 7);
    println!("{}", i64::MAX.to_string());
    println!("{}", (-17i32).to_string() + &3.5f64.to_string());
    println!("{}", char::from(65u8));
    println!("{}", u8::from(b'a'));
    println!("{}", '9' as u8 - b'0');
    println!("{}", rot13("Hello, World!"));
    println!("{}", rot13(&rot13("Round Trip")));
    println!("{:?}", i32::from_str_radix("-80000000", 16));
    println!("{:?}", i32::from_str_radix("80000000", 16).is_err());
    println!("{} {}", u32::MAX.leading_zeros(), usize::BITS);
    let celsius = [-40.0, 0.0, 37.0, 100.0];
    for c in celsius {
        println!("{:>7.1}C = {:>7.1}F", c, c * 1.8 + 32.0);
    }
    let table: Vec<String> = (1..=5u32).map(|i| format!("{:>3}^2={:<4}|", i, i.pow(2))).collect();
    println!("{}", table.concat());
    println!("{:>10}", format!("{:.2}%", 45.678));
    println!("{:?}", (1.0f64, -0.0f32, f64::INFINITY));
    println!("{:?}", vec![0.5f32, 1.0, 1e-8]);
    println!("{:.2?}", vec![0.555f64, 1.0]);
    println!("{:#?}", (1u8, 2.5f64));
}
