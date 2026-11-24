use std::fmt;

#[derive(Debug)]
enum AppError {
    NotFound { id: u32 },
    Invalid(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::NotFound { id } => write!(f, "item {} not found", id),
            AppError::Invalid(why) => write!(f, "invalid: {}", why),
        }
    }
}

impl std::error::Error for AppError {}

fn find(id: u32) -> Result<&'static str, AppError> {
    match id {
        1 => Ok("apple"),
        2 => Ok("banana"),
        _ => Err(AppError::NotFound { id }),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", find(1)?);
    println!("{}", find(2)?);
    let _ = AppError::Invalid("unused".to_string());
    let n: i32 = "42".parse()?;
    println!("parsed {}", n);
    println!("{}", find(7)?);
    println!("never printed");
    Ok(())
}
