fn main() {
    // positional and named arguments
    println!("{} days in {}", 31, "January");
    println!("{0}, this is {1}. {1}, this is {0}.", "Alice", "Bob");
    println!("{subject} {verb} {object}", object = "the lazy dog", subject = "the quick brown fox", verb = "jumps over");
    let city = "Lisbon";
    let pop = 545_923;
    println!("{city} has {pop} people");

    // width, fill, and alignment
    println!("[{:>10}]", "right");
    println!("[{:<10}]", "left");
    println!("[{:^10}]", "center");
    println!("[{:*^12}]", "star");
    println!("[{:->8}] [{:_<8}]", 42, 42);
    println!("[{:>width$}] [{:>1$}]", "dyn", 7, width = 6);
    let w = 9;
    println!("[{:^w$}]", "local");

    // numbers: sign, zero padding, precision
    println!("[{:+}] [{:+}] [{:05}] [{:<05}]", 17, -17, 42, 42);
    println!("[{:.0}] [{:.2}] [{:8.3}] [{:<8.1}] [{:+.4}]", 2.5f64, 3.14159, 2.71828, 9.99, 1.0f32 / 3.0);
    println!("[{:.*}] [{:.prec$}]", 3, 1.23456789, 9.87654321, prec = 1);
    println!("[{:e}] [{:E}] [{:.2e}]", 1500.0, 0.00025, 6.02e23);
    println!("[{:x}] [{:X}] [{:#x}] [{:o}] [{:#o}] [{:b}] [{:#b}]", 254, 254, 254, 8, 8, 5, 5);
    println!("[{:#010x}] [{:08b}] [{:>#6x}]", 3054, 19, 31);

    // Debug vs Display, and escaping
    println!("display: {}  debug: {:?}", "tab\there", "tab\there");
    println!("chars: {} {:?} {:?}", 'q', 'q', '\n');
    println!("{{literal braces}} {{{}}}", "inside");
    println!("unicode {:?} len {}", "héllo", "héllo".len());
    let tuple = (1, "two", 3.0, ['a', 'b']);
    println!("{:?}", tuple);
    println!("{:#?}", tuple);
    println!("{:?} {:?} {:?}", Some(5), None::<i32>, Ok::<u8, String>(1));

    // format! and friends build Strings
    let s = format!("{:>4}|{:<4}|{:^4}", 1, 2, 3);
    println!("{} (len {})", s, s.len());
    let mut report = String::new();
    for (i, item) in ["apple", "fig", "kiwi"].iter().enumerate() {
        report += &format!("{:02}. {:<6}|", i + 1, item);
    }
    println!("{}", report);
    print!("print! without newline, ");
    print!("{}\n", "then a newline");
    let pi = std::f64::consts::PI;
    println!("{pi:.3} {pi:10.5} {pi:<10.2}| {:?}", pi as f32);
}
