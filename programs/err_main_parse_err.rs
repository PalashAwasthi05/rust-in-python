use std::num::ParseIntError;

fn double(s: &str) -> Result<i32, ParseIntError> {
    let n = s.parse::<i32>()?;
    Ok(n * 2)
}

fn main() -> Result<(), ParseIntError> {
    println!("{}", double("21")?);
    print!("partial line without newline ");
    println!("{}", double("4x")?);
    Ok(())
}
