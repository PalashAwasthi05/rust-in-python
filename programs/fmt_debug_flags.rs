use std::collections::BTreeMap;

#[derive(Debug)]
struct P {
    x: i32,
    y: f64,
    s: &'static str,
    b: bool,
}

fn main() {
    println!("{:5?}", vec![1, 22, 333]);
    println!("{:<4?}|", (1, 2));
    println!("{:03?}", [7, -7]);
    println!("{:+?}", vec![1, -1]);
    println!("{:.2?}", (1.0, 2.5f32));
    println!("{:8.3?}", Some(3.14159));
    println!("{:5?}", P { x: 1, y: 2.0, s: "s", b: true });
    println!("{:#6?}", vec![1, 2]);
    println!("{:x?}", vec![255, 16]);
    println!("{:X?}", P { x: 255, y: 1.0, s: "s", b: false });
    println!("{:#x?}", vec![10]);
    println!("{:#X?}", Some(171));
    println!("{:02x?}", [1u8, 2, 255]);
    println!("{:?}", -0.0);
    println!("{:5?}|", "s");
    println!("{:5?}|", 'c');
    println!("{:5?}|", true);
    println!("{:5?}|", ());
    let mut m = BTreeMap::new();
    m.insert("k", 1.5);
    println!("{:6.2?}", m);
    println!("{:>4?}", Some("str"));
    println!("{:e}", 1500);
    println!("{:?}", u8::MAX);
    println!("{:3?}|", None::<i32>);
}
