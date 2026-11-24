use std::convert::TryFrom;
use std::fmt;

#[derive(Debug, Clone, Copy)]
struct Meters(f64);
#[derive(Debug, Clone, Copy)]
struct Feet(f64);

impl From<Feet> for Meters {
    fn from(f: Feet) -> Meters {
        Meters(f.0 * 0.3048)
    }
}

impl From<Meters> for Feet {
    fn from(m: Meters) -> Feet {
        Feet(m.0 / 0.3048)
    }
}

#[derive(Debug)]
enum ParseError {
    Empty,
    BadNumber(String),
    Negative(i64),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Empty => write!(f, "empty input"),
            ParseError::BadNumber(s) => write!(f, "bad number: {}", s),
            ParseError::Negative(n) => write!(f, "negative: {}", n),
        }
    }
}

impl From<std::num::ParseIntError> for ParseError {
    fn from(e: std::num::ParseIntError) -> Self {
        ParseError::BadNumber(e.to_string())
    }
}

#[derive(Debug)]
struct Even(u32);

impl TryFrom<i64> for Even {
    type Error = ParseError;
    fn try_from(v: i64) -> Result<Self, Self::Error> {
        if v < 0 {
            Err(ParseError::Negative(v))
        } else if v % 2 != 0 {
            Err(ParseError::BadNumber(format!("{} is odd", v)))
        } else {
            Ok(Even(v as u32))
        }
    }
}

fn parse_even(s: &str) -> Result<Even, ParseError> {
    let s = s.trim();
    if s.is_empty() {
        return Err(ParseError::Empty);
    }
    let n: i64 = s.parse()?;
    let e = Even::try_from(n)?;
    Ok(e)
}

struct Name(String);

impl From<&str> for Name {
    fn from(s: &str) -> Self {
        Name(s.to_string())
    }
}

fn greet<N: Into<Name>>(n: N) -> String {
    let name: Name = n.into();
    format!("hi {}", name.0)
}

fn main() {
    let m: Meters = Feet(10.0).into();
    println!("{:.3}", m.0);
    let f = Feet::from(Meters(1.0));
    println!("{:.3}", f.0);
    let round: Feet = Meters::from(Feet(3.0)).into();
    println!("{:.3}", round.0);
    for s in ["42", " 8 ", "", "x1", "-4", "7"] {
        match parse_even(s) {
            Ok(e) => println!("ok {:?}", e),
            Err(e) => println!("error: {}", e),
        }
    }
    println!("{}", greet("ann"));
    println!("{}", greet(Name("bo".to_string())));
    let big: i64 = i64::from(7u8) + i64::from(-3i32);
    println!("{}", big);
    let small = u8::try_from(300i32);
    println!("{:?}", small.is_err());
    let ok: Result<u8, _> = 200i32.try_into();
    println!("{:?}", ok);
    let s: String = 'c'.into();
    let v: Vec<u8> = "hey".into();
    let b: Box<str> = "boxed".into();
    println!("{} {:?} {}", s, v, b);
    let opt: Option<i32> = 5.into();
    println!("{:?}", opt);
    let f: f64 = 3i32.into();
    println!("{}", f);
}
