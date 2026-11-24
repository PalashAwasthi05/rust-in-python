struct Row {
    name: &'static str,
    qty: u32,
    price: f64,
}

fn main() {
    let rows = [
        Row { name: "Espresso", qty: 3, price: 2.5 },
        Row { name: "Cappuccino", qty: 1, price: 3.75 },
        Row { name: "Blueberry muffin", qty: 2, price: 2.2 },
        Row { name: "Tea", qty: 10, price: 1.0 },
    ];
    let width = 36;
    println!("{}", "=".repeat(width));
    println!("{:^36}", "CORNER CAFE");
    println!("{:-^36}", " receipt ");
    println!("{:<18}{:>5}{:>13}", "Item", "Qty", "Total");
    let mut subtotal = 0.0;
    for r in &rows {
        let line = r.qty as f64 * r.price;
        subtotal += line;
        let name = if r.name.len() > 16 { format!("{}..", &r.name[..14]) } else { r.name.to_string() };
        println!("{:<18}{:>5}{:>13.2}", name, r.qty, line);
    }
    let tax = subtotal * 0.0825;
    println!("{:>36}", "--------");
    println!("{:<23}{:>13.2}", "Subtotal", subtotal);
    println!("{:<23}{:>13.2}", "Tax 8.25%", tax);
    println!("{:<23}{:>13.2}", "TOTAL", subtotal + tax);
    println!("{}", "=".repeat(width));

    let n = 255;
    println!("dec={} hex={:x} HEX={:X} oct={:o} bin={:b}", n, n, n, n, n);
    println!("alt: {:#x} {:#o} {:#b} {:#010b}", n, n, n, 5);
    println!("padded: [{:05}] [{:+}] [{:+.1}] [{:<5}] [{:>5}] [{:^5}]", 42, 42, -3.14159, 7, 7, 7);
    println!("fill: [{:*<8}] [{:_>8}] [{:.^9}]", "ab", "cd", "mid");
    let w = 10;
    let p = 3;
    println!("dynamic: [{:>w$.p$}] [{:>1$}] [{:.*}]", 2.718281828, 6, 2, 1.23456, w = w, p = p);
    println!("sci: {:e} {:.2e}", 1234.5, 0.000321);
    let pi = std::f64::consts::PI;
    println!("{pi:.0} {pi:.2} {pi:8.4}|");
    println!("debug: {:?} {:?} {:?}", "quote\"d", 'x', Some(1.5));
    let nested = vec![(1, "one"), (2, "two")];
    println!("pretty:\n{:#?}", nested);
    println!("{{braces}} and {:>4}%", 99);
}
