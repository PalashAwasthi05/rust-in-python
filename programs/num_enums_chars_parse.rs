use std::num::NonZeroU32;

#[derive(Debug, Clone, Copy)]
enum Color { Red, Green = 10, Blue }

#[repr(i8)]
#[derive(Debug, Clone, Copy)]
enum Signed { Neg = -3, Zero = 0, Pos = 100 }

#[repr(u8)]
enum Op { Nop = 0x90, Ret = 0xC3 }

fn main() {
    println!("{} {} {}", Color::Red as i32, Color::Green as u8, Color::Blue as i64);
    println!("{} {} {}", Signed::Neg as i8, Signed::Neg as u8, Signed::Pos as i32);
    println!("{} {}", Op::Nop as u8, Op::Ret as u32);
    println!("{:?}", Signed::Zero as i8);
    let n = NonZeroU32::new(5).unwrap();
    println!("{} {:?}", n.get(), NonZeroU32::new(0));
    let z = std::num::NonZeroUsize::new(12).map(|v| v.get() * 2);
    println!("{:?} {}", z, n);
    println!("{:?}", "1e400".parse::<f64>());
    println!("{:?}", "1e-400".parse::<f64>());
    println!("{:?}", "1.".parse::<f64>());
    println!("{:?}", ".5".parse::<f64>());
    println!("{:?}", "NaN".parse::<f64>().map(|v| v.is_nan()));
    println!("{:?}", "infinity".parse::<f64>());
    println!("{:?}", "-inf".parse::<f32>());
    println!("{:?}", "1_000".parse::<i32>().is_err());
    println!("{:?}", "0x10".parse::<i32>().is_err());
    println!("{:?}", "3.4e38".parse::<f32>());
    println!("{:?}", "3.5e38".parse::<f32>());
    println!("{:?}", "16777217".parse::<f32>());
    println!("{}", 233u8 as char);
    println!("{:?}", (b'a'..=b'e').map(|b| b as char).collect::<String>());
    println!("{:?}", ('a'..'f').map(|c| c as u32 - 'a' as u32).collect::<Vec<_>>());
    println!("{}", 1e-5f32);
    println!("{}", 1e-5f64);
    println!("{}", 123456789.0f32);
    println!("{}", 0.1f32 * 0.1f32);
    let a = 0.1 + 0.2;
    println!("{}", (a - 0.3f64).abs() < f64::EPSILON);
    let squares: Vec<f64> = (1..=4).map(|i| (i as f64).powi(2)).collect();
    println!("{:?}", squares);
    let e: u32 = 5;
    println!("{}", 3usize.pow(e));
    println!("{}", usize::MAX / 2);
    println!("{}", 'A'.is_ascii_uppercase() as u8 + b'0');
    println!("{:?}", '7'.to_digit(10).map(|d| d * 3));
    println!("{}", ('Z' as u8 - b'A') as usize);
    println!("{}", char::from_u32(0x41).unwrap());
    println!("{}", std::char::from_digit(11, 16).unwrap());
    println!("{}", u32::from('é'));
    println!("{:?}", u8::try_from('é'));
    println!("{:?}", u8::try_from('ÿ'));
    println!("{:?}", char::try_from(65u32));
    println!("{}", char::from(0x263Au16 as u8));
}
