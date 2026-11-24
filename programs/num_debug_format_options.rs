#[derive(Debug)]
struct P { x: f64, y: u8 }

fn main() {
    println!("{:5?}|", vec![1, 22, 333]);
    println!("{:<4?}|", vec!["a", "bc"]);
    println!("{:.1?}", Some(1.25));
    println!("{:.1?}", P { x: 2.25, y: 7 });
    println!("{:x?}", (255u8, 16i32));
    println!("{:X?}", vec![0xabu8, 0xcd]);
    println!("{:02x?}", vec![1u8, 0x80, 0xff]);
    println!("{:04?}", [1.5, -2.0]);
    println!("{:+?}", vec![1, -1]);
    println!("{:?}", 255u8);
    println!("{:x?}", 255u8);
    println!("{:5?}|", 7);
    println!("{:.3?}", 1.0f32 / 3.0);
    println!("{:#.1?}", vec![0.25f64]);
    println!("{:.2?}", (1.0, "s", 'c'));
    println!("{:6.2?}|", [0.125f32]);
    println!("{:5?}|{:5?}|{:6?}|{:<4?}|", "a", 'c', vec![true], vec![String::from("s")]);
    println!("{:>5?}|{:^7?}|", Some("x"), ('q', 1));
    println!("{:6?}|{:6?}|{:6?}|", vec![true], [false], true);
    println!("{:6}|", true);
}
