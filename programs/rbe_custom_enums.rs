#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq)]
enum Number { Zero, One, Two }

#[derive(Debug, Clone, Copy)]
enum Color { Red = 0xff0000, Green = 0x00ff00, Blue = 0x0000ff }

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[repr(u8)]
enum Level { Trace = 1, Debug, Info = 10, Warn, Error = 50 }

enum WebEvent {
    PageLoad,
    PageUnload,
    KeyPress(char),
    Paste(String),
    Click { x: i64, y: i64 },
}

fn inspect(event: WebEvent) -> String {
    match event {
        WebEvent::PageLoad => "page loaded".to_string(),
        WebEvent::PageUnload => "page unloaded".to_string(),
        WebEvent::KeyPress(c) => format!("pressed '{}'", c),
        WebEvent::Paste(s) => format!("pasted \"{}\"", s),
        WebEvent::Click { x, y } => format!("clicked at x={}, y={}", x, y),
    }
}

enum VeryVerboseEnumOfThingsToDoWithNumbers { Add, Subtract }
type Operations = VeryVerboseEnumOfThingsToDoWithNumbers;

impl VeryVerboseEnumOfThingsToDoWithNumbers {
    fn run(&self, x: i32, y: i32) -> i32 {
        match self {
            Self::Add => x + y,
            Self::Subtract => x - y,
        }
    }
}

impl Level {
    fn from_u8(v: u8) -> Option<Level> {
        use Level::*;
        [Trace, Debug, Info, Warn, Error].into_iter().find(|l| *l as u8 == v)
    }
}

fn main() {
    println!("zero is {}", Number::Zero as i32);
    println!("one is {}", Number::One as i32);
    println!("two as u8 is {}", Number::Two as u8);
    println!("roses are #{:06x}", Color::Red as i32);
    println!("violets are #{:06x}", Color::Blue as u32);
    println!("green as i64 {}", Color::Green as i64);
    for l in [Level::Trace, Level::Debug, Level::Info, Level::Warn, Level::Error] {
        println!("{:?} = {}", l, l as u8);
    }
    println!("Warn > Info: {}, Debug < Trace: {}", Level::Warn > Level::Info, Level::Debug < Level::Trace);
    println!("from_u8(11) = {:?}, from_u8(3) = {:?}", Level::from_u8(11), Level::from_u8(3));
    println!("size of Level: {} byte", std::mem::size_of::<Level>());

    let events = vec![
        WebEvent::KeyPress('x'),
        WebEvent::Paste("my text".to_owned()),
        WebEvent::Click { x: 20, y: 80 },
        WebEvent::PageLoad,
        WebEvent::PageUnload,
    ];
    for e in events {
        println!("{}", inspect(e));
    }

    let x = Operations::Add;
    let y = Operations::Subtract;
    println!("alias add {} subtract {}", x.run(7, 5), y.run(7, 5));

    use Number::{One, Two};
    let n = Two;
    let word = match n { Number::Zero => "nothing", One => "single", Two => "double" };
    println!("{:?} is {}", n, word);
    println!("equal? {}", One == Number::One);
}
