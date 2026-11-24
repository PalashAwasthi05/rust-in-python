use std::error::Error;
use std::fmt;

#[derive(Debug)]
struct NegativeError(i32);

impl fmt::Display for NegativeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} is negative", self.0)
    }
}

impl Error for NegativeError {}

fn parse_positive(s: &str) -> Result<i32, Box<dyn Error>> {
    let n: i32 = s.parse()?;
    if n < 0 {
        return Err(Box::new(NegativeError(n)));
    }
    if n == 0 {
        return Err("zero is not allowed".into());
    }
    if n > 1000 {
        return Err(format!("{} is too large", n).into());
    }
    Ok(n)
}

fn sum_all(items: &[&str]) -> Result<i32, Box<dyn Error>> {
    let mut total = 0;
    for item in items {
        total += parse_positive(item)?;
    }
    let half: f64 = "2.5".parse()?;
    Ok(total + half as i32)
}

fn describe(e: &Box<dyn Error>) -> String {
    if let Some(neg) = e.downcast_ref::<NegativeError>() {
        format!("negative error holding {}", neg.0)
    } else if e.downcast_ref::<std::num::ParseIntError>().is_some() {
        format!("parse int error: {}", e)
    } else {
        format!("other error: {}", e)
    }
}

fn main() {
    for input in ["12", "-4", "0", "abc", "5000", ""] {
        match parse_positive(input) {
            Ok(n) => println!("{:?} -> {}", input, n),
            Err(e) => println!("{:?} -> {}", input, describe(&e)),
        }
    }
    println!("{:?}", sum_all(&["1", "2", "3"]).ok());
    match sum_all(&["1", "x", "3"]) {
        Ok(_) => println!("unexpected"),
        Err(e) => println!("sum failed: {}", e.to_string()),
    }
    let errors: Vec<Box<dyn Error>> = vec![
        Box::new(NegativeError(-7)),
        "plain".into(),
        Box::new("1.2.3".parse::<f64>().unwrap_err()),
    ];
    for e in &errors {
        println!("[{}] {:?} {}", e, e, e.source().is_none());
    }
}
