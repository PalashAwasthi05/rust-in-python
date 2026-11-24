use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
struct EvenNumber(i32);

impl TryFrom<i32> for EvenNumber {
    type Error = String;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value % 2 == 0 { Ok(EvenNumber(value)) } else { Err(format!("{} is odd", value)) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Weekday { Mon, Tue, Wed, Thu, Fri, Sat, Sun }

impl TryFrom<u8> for Weekday {
    type Error = u8;
    fn try_from(n: u8) -> Result<Self, u8> {
        use Weekday::*;
        [Mon, Tue, Wed, Thu, Fri, Sat, Sun].get(n as usize).copied().ok_or(n)
    }
}

impl fmt::Display for Weekday {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = format!("{:?}", self);
        write!(f, "{}", s.to_uppercase())
    }
}

impl FromStr for Weekday {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        (0u8..7).filter_map(|i| Weekday::try_from(i).ok()).find(|d| d.to_string().eq_ignore_ascii_case(s)).ok_or_else(|| format!("not a day: {}", s))
    }
}

struct Circle { radius: i32 }

impl fmt::Display for Circle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Circle of radius {}", self.radius)
    }
}

#[derive(Debug)]
struct Rgb { r: u8, g: u8, b: u8 }

impl FromStr for Rgb {
    type Err = std::num::ParseIntError;
    fn from_str(hex: &str) -> Result<Self, Self::Err> {
        let h = hex.trim_start_matches('#');
        Ok(Rgb { r: u8::from_str_radix(&h[0..2], 16)?, g: u8::from_str_radix(&h[2..4], 16)?, b: u8::from_str_radix(&h[4..6], 16)? })
    }
}

fn main() {
    assert_eq!(EvenNumber::try_from(8), Ok(EvenNumber(8)));
    println!("try_from(8) = {:?}", EvenNumber::try_from(8));
    println!("try_from(5) = {:?}", EvenNumber::try_from(5));
    let result: Result<EvenNumber, String> = 8i32.try_into();
    println!("8.try_into() = {:?}", result);
    let result: Result<EvenNumber, _> = 5i32.try_into();
    println!("5.try_into() = {:?}", result);

    for n in [0u8, 4, 6, 9] {
        match Weekday::try_from(n) {
            Ok(d) => println!("{} -> {} ({:?})", n, d, d),
            Err(bad) => println!("{} -> invalid day {}", n, bad),
        }
    }
    let narrow: Result<u8, _> = 256u16.try_into();
    let fine: Result<u8, _> = 255u16.try_into();
    let neg: Result<u32, _> = (-1i64).try_into();
    println!("u16->u8 {:?} {:?}, i64->u32 err? {}", narrow.is_err(), fine, neg.is_err());
    let idx: usize = 42u64.try_into().unwrap();
    println!("u64 -> usize: {}", idx);

    let circle = Circle { radius: 6 };
    println!("{}", circle.to_string());
    let label = 3.5f64.to_string() + &'!'.to_string() + &true.to_string();
    println!("to_string concat: {}", label);

    let parsed: i32 = "5".parse().unwrap();
    let turbo = "10".parse::<i32>().unwrap();
    println!("sum = {}", parsed + turbo);
    println!("{:?} {:?} {:?}", "3.75".parse::<f64>(), "-0".parse::<i8>(), "x1".parse::<u32>().is_err());
    println!("bool {:?} char {:?}", "true".parse::<bool>(), "z".parse::<char>());
    println!("{:?} {:?}", "wed".parse::<Weekday>(), "Funday".parse::<Weekday>());
    println!("{:?}", "#1e90ff".parse::<Rgb>());
    println!("{:?}", "zz0000".parse::<Rgb>().map_err(|e| e.to_string()));
    let days: Result<Vec<Weekday>, _> = "mon fri sun".split(' ').map(str::parse::<Weekday>).collect();
    println!("{:?}", days);
}
