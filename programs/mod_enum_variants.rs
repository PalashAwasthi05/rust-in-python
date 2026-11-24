// Enum variant imports with use Enum::* and use Enum::{A, B}.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Color {
    Red,
    Green,
    Blue,
    Custom(u8, u8, u8),
}

use Color::*;

mod traffic {
    #[derive(Debug)]
    pub enum Light {
        Stop,
        Caution,
        Go { seconds: u32 },
    }

    pub use self::Light::{Caution, Go, Stop};

    pub fn next(l: &Light) -> Light {
        match l {
            Stop => Go { seconds: 30 },
            Go { .. } => Caution,
            Caution => Stop,
        }
    }
}

fn hex(c: Color) -> String {
    match c {
        Red => "#f00".to_string(),
        Green => "#0f0".to_string(),
        Blue => "#00f".to_string(),
        Custom(r, g, b) => format!("#{:02x}{:02x}{:02x}", r, g, b),
    }
}

fn main() {
    for c in [Red, Green, Blue, Custom(1, 171, 255)] {
        println!("{:?} -> {}", c, hex(c));
    }
    println!("{}", Red == Color::Red);
    use traffic::Light::{self, *};
    let mut l: Light = Stop;
    for _ in 0..4 {
        l = traffic::next(&l);
        println!("{:?}", l);
    }
    if let Go { seconds } = traffic::next(&traffic::Stop) {
        println!("go for {}", seconds);
    }
    use std::option::Option::{None as Nothing, Some as Just};
    let x: Option<i32> = Just(3);
    match x {
        Just(n) => println!("just {}", n),
        Nothing => println!("nothing"),
    }
}
