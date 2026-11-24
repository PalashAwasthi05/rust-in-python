// Chapter 9: recoverable errors with Result, the ? operator, and validated types.
use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

#[derive(Debug)]
enum ConfigError {
    Missing(String),
    BadNumber(String, ParseIntError),
    OutOfRange(i64),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::Missing(key) => write!(f, "missing key `{key}`"),
            ConfigError::BadNumber(key, e) => write!(f, "key `{key}` is not a number: {e}"),
            ConfigError::OutOfRange(n) => write!(f, "{n} is out of range"),
        }
    }
}

impl Error for ConfigError {}

impl From<ParseIntError> for ConfigError {
    fn from(e: ParseIntError) -> Self {
        ConfigError::BadNumber(String::from("?"), e)
    }
}

fn lookup<'a>(pairs: &'a [(&'a str, &'a str)], key: &str) -> Result<&'a str, ConfigError> {
    for (k, v) in pairs {
        if *k == key {
            return Ok(v);
        }
    }
    Err(ConfigError::Missing(key.to_string()))
}

fn read_port(pairs: &[(&str, &str)]) -> Result<u16, ConfigError> {
    let raw = lookup(pairs, "port")?;
    let n: i64 = raw.trim().parse().map_err(|e| ConfigError::BadNumber("port".into(), e))?;
    if !(1..=65535).contains(&n) {
        return Err(ConfigError::OutOfRange(n));
    }
    Ok(n as u16)
}

fn read_retries(pairs: &[(&str, &str)]) -> Result<u32, ConfigError> {
    let raw = lookup(pairs, "retries").unwrap_or("3");
    Ok(raw.parse::<u32>()?)
}

fn read_username(source: Option<&str>) -> Result<String, String> {
    let name = source.ok_or_else(|| String::from("no username file"))?;
    if name.is_empty() {
        Err(String::from("username file was empty"))
    } else {
        Ok(name.to_uppercase())
    }
}

fn last_char_of_first_line(text: &str) -> Option<char> {
    text.lines().next()?.chars().last()
}

fn sum_all(inputs: &[&str]) -> Result<i32, Box<dyn Error>> {
    let mut total = 0;
    for s in inputs {
        let n: i32 = s.parse()?;
        total += n;
    }
    if total < 0 {
        return Err(Box::new(ConfigError::OutOfRange(total as i64)));
    }
    if total > 1000 {
        return Err("total too large".into());
    }
    Ok(total)
}

pub struct Guess {
    value: i32,
}

impl Guess {
    pub fn new(value: i32) -> Result<Guess, String> {
        if value < 1 || value > 100 {
            return Err(format!("Guess value must be between 1 and 100, got {value}."));
        }
        Ok(Guess { value })
    }

    pub fn value(&self) -> i32 {
        self.value
    }
}

fn main() {
    let good = [("host", "localhost"), ("port", " 8080 ")];
    let bad_number = [("port", "eighty")];
    let out_of_range = [("port", "70000"), ("retries", "x")];
    let empty: [(&str, &str); 0] = [];

    for (label, cfg) in [("good", &good[..]), ("bad", &bad_number[..]), ("range", &out_of_range[..]), ("empty", &empty[..])] {
        match read_port(cfg) {
            Ok(port) => println!("{label}: port {port}"),
            Err(e) => println!("{label}: error: {e}"),
        }
        match read_retries(cfg) {
            Ok(n) => println!("{label}: retries {n}"),
            Err(e) => println!("{label}: retries error: {e} / {e:?}"),
        }
    }

    let port = read_port(&good).unwrap_or_else(|e| {
        println!("problem: {e}");
        80
    });
    println!("port via unwrap_or_else: {port}");
    let fallback = read_port(&empty).unwrap_or_default();
    println!("fallback: {fallback}");

    println!("{:?}", read_username(Some("ferris")));
    println!("{:?}", read_username(Some("")));
    println!("{:?}", read_username(None));

    println!("{:?}", last_char_of_first_line("Hello, world\nHow are you"));
    println!("{:?}", last_char_of_first_line(""));
    println!("{:?}", last_char_of_first_line("\nhi"));

    for inputs in [&["1", "2", "3"][..], &["4", "x"][..], &["-5", "1"][..], &["999", "2"][..]] {
        match sum_all(inputs) {
            Ok(total) => println!("sum = {total}"),
            Err(e) => println!("sum failed: {e}"),
        }
    }

    for v in [0, 1, 50, 100, 101] {
        match Guess::new(v) {
            Ok(g) => println!("accepted guess {}", g.value()),
            Err(msg) => println!("{msg}"),
        }
    }

    let parsed: Result<Vec<i32>, _> = "1,2,3".split(',').map(str::parse::<i32>).collect();
    println!("{:?}", parsed);
    let parsed: Result<Vec<i32>, _> = "1,two,3".split(',').map(|s| s.parse::<i32>()).collect();
    println!("{:?}", parsed);
    let total: i32 = ["7", "oops", "8"].iter().filter_map(|s| s.parse::<i32>().ok()).sum();
    println!("total of valid = {total}");
    let r: Result<i32, String> = Err("boom".to_string());
    println!("{} {} {:?}", r.is_err(), r.clone().unwrap_or(-1), r.as_ref().err());
    let x: Result<u8, _> = "300".parse::<u8>();
    if let Err(e) = &x {
        println!("u8 parse error: {e}");
    }
    let n = "12".parse::<u8>().expect("should parse");
    println!("expect gave {n}");
}
