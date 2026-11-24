use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone, Copy)]
struct Point { x: i32, y: i32 }

#[derive(Debug, PartialEq)]
enum ParsePointError { Format, Number(String) }

impl fmt::Display for ParsePointError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParsePointError::Format => write!(f, "expected (x, y)"),
            ParsePointError::Number(s) => write!(f, "bad number {:?}", s),
        }
    }
}

impl FromStr for Point {
    type Err = ParsePointError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner = s.trim().strip_prefix('(').and_then(|r| r.strip_suffix(')')).ok_or(ParsePointError::Format)?;
        let (a, b) = inner.split_once(',').ok_or(ParsePointError::Format)?;
        let num = |t: &str| t.trim().parse::<i32>().map_err(|_| ParsePointError::Number(t.trim().to_string()));
        Ok(Point { x: num(a)?, y: num(b)? })
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Version { major: u32, minor: u32, patch: u32 }

impl FromStr for Version {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.trim_start_matches('v').split('.').collect();
        if parts.len() != 3 { return Err(format!("need 3 parts in {:?}", s)); }
        let n: Result<Vec<u32>, _> = parts.iter().map(|p| p.parse::<u32>()).collect();
        match n {
            Ok(v) => Ok(Version { major: v[0], minor: v[1], patch: v[2] }),
            Err(e) => Err(format!("{:?}: {}", s, e)),
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Debug)]
enum Color { Red, Green, Blue, Hex(u32) }

impl FromStr for Color {
    type Err = ();
    fn from_str(s: &str) -> Result<Color, ()> {
        match s.to_ascii_lowercase().as_str() {
            "red" => Ok(Color::Red),
            "green" => Ok(Color::Green),
            "blue" => Ok(Color::Blue),
            h if h.starts_with('#') && h.len() == 7 => u32::from_str_radix(&h[1..], 16).map(Color::Hex).map_err(|_| ()),
            _ => Err(()),
        }
    }
}

fn main() {
    for s in ["(3, 4)", " (-10,7) ", "3,4", "(a, 2)", "(1; 2)"] {
        match s.parse::<Point>() {
            Ok(p) => println!("{:<10} -> {:?}", s, p),
            Err(e) => println!("{:<10} -> error: {}", s, e),
        }
    }
    let mut versions: Vec<Version> = ["1.10.0", "v1.2.3", "0.9.12", "1.2.10"].iter().filter_map(|s| s.parse().ok()).collect();
    versions.sort();
    let shown: Vec<String> = versions.iter().map(|v| v.to_string()).collect();
    println!("sorted versions: {}", shown.join(" < "));
    println!("{:?}", "1.2".parse::<Version>());
    println!("{:?}", "1.x.3".parse::<Version>());
    let colors: Vec<Result<Color, ()>> = "Red blue #ff8800 purple #12".split(' ').map(str::parse).collect();
    println!("{:?}", colors);
    let total: i32 = "(1,2) (3,4) (5,6)".split(' ').map(|t| t.parse::<Point>().unwrap()).map(|p| p.x * p.y).sum();
    println!("sum of x*y: {}", total);
}
