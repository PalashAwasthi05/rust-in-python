// A module-level Result alias, an error enum with From impls, and ? across modules.
mod errors {
    use std::fmt;

    #[derive(Debug)]
    pub enum AppError {
        Parse(std::num::ParseIntError),
        Range(i64),
    }

    impl fmt::Display for AppError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                AppError::Parse(e) => write!(f, "parse error: {}", e),
                AppError::Range(n) => write!(f, "{} out of range", n),
            }
        }
    }

    impl std::error::Error for AppError {}

    impl From<std::num::ParseIntError> for AppError {
        fn from(e: std::num::ParseIntError) -> Self {
            AppError::Parse(e)
        }
    }

    pub type Result<T> = std::result::Result<T, AppError>;
}

mod parse {
    use crate::errors::{AppError, Result};

    pub fn small(s: &str) -> Result<i64> {
        let n: i64 = s.trim().parse()?;
        if !(0..100).contains(&n) {
            return Err(AppError::Range(n));
        }
        Ok(n)
    }

    pub fn sum(items: &[&str]) -> Result<i64> {
        let mut total = 0;
        for i in items {
            total += small(i)?;
        }
        Ok(total)
    }
}

fn boxed(s: &str) -> Result<i64, Box<dyn std::error::Error>> {
    Ok(parse::small(s)? * 2)
}

fn main() {
    for s in ["42", " 7 ", "x", "500"] {
        match parse::small(s) {
            Ok(n) => println!("ok {}", n),
            Err(e) => println!("error: {}", e),
        }
    }
    println!("{:?}", parse::sum(&["1", "2", "3"]).unwrap());
    println!("{}", parse::sum(&["1", "200"]).unwrap_err());
    println!("{:?}", boxed("4"));
    match boxed("-1") {
        Err(e) => println!("boxed {}", e),
        Ok(_) => {}
    }
}
