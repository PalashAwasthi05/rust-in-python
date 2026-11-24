use std::num::ParseIntError;

type AliasedResult<T> = std::result::Result<T, ParseIntError>;

fn multiply_match(first: &str, second: &str) -> Result<i32, ParseIntError> {
    match first.parse::<i32>() {
        Ok(first_number) => match second.parse::<i32>() {
            Ok(second_number) => Ok(first_number * second_number),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

fn multiply_combinators(first: &str, second: &str) -> AliasedResult<i32> {
    first.parse::<i32>().and_then(|a| second.parse::<i32>().map(|b| a * b))
}

fn multiply_early(first: &str, second: &str) -> AliasedResult<i32> {
    let first_number = match first.parse::<i32>() {
        Ok(n) => n,
        Err(e) => return Err(e),
    };
    let second_number = match second.parse::<i32>() {
        Ok(n) => n,
        Err(e) => return Err(e),
    };
    Ok(first_number * second_number)
}

fn multiply_question(first: &str, second: &str) -> AliasedResult<i32> {
    let a = first.parse::<i32>()?;
    let b = second.parse::<i32>()?;
    Ok(a * b)
}

fn print(result: AliasedResult<i32>) {
    match result {
        Ok(n) => println!("n is {}", n),
        Err(e) => println!("Error: {}", e),
    }
}

#[derive(Debug, PartialEq)]
enum AgeError { Negative(i32), TooOld(i32), Parse(String) }

fn parse_age(s: &str) -> Result<u8, AgeError> {
    s.trim()
        .parse::<i32>()
        .map_err(|e| AgeError::Parse(e.to_string()))
        .and_then(|n| if n < 0 { Err(AgeError::Negative(n)) } else { Ok(n) })
        .and_then(|n| if n > 150 { Err(AgeError::TooOld(n)) } else { Ok(n as u8) })
}

fn main() {
    print(multiply_match("10", "2"));
    print(multiply_match("t", "2"));
    print(multiply_combinators("10", "2"));
    print(multiply_combinators("10", "x"));
    print(multiply_early("7", "6"));
    print(multiply_question("3", "-4"));
    print(multiply_question("", "1"));

    for s in ["42", " 7 ", "-3", "200", "abc"] {
        println!("parse_age({:?}) = {:?}", s, parse_age(s));
    }

    let ok: Result<i32, String> = Ok(3);
    let err: Result<i32, String> = Err("boom".into());
    println!("map: {:?} {:?}", ok.clone().map(|v| v * 2), err.clone().map(|v| v * 2));
    println!("map_err: {:?}", err.clone().map_err(|e| e.len()));
    println!("unwrap_or: {} {}", ok.clone().unwrap_or(0), err.clone().unwrap_or(0));
    println!("unwrap_or_else: {}", err.clone().unwrap_or_else(|e| e.len() as i32 * -1));
    println!("or_else: {:?}", err.clone().or_else(|_| Ok::<i32, String>(99)));
    println!("ok(): {:?} {:?}, err(): {:?}", ok.clone().ok(), err.clone().ok(), err.clone().err());
    println!("is_ok {} is_err {}", ok.is_ok(), err.is_err());
    println!("and: {:?}", ok.clone().and(Ok::<&str, String>("next")));
    println!("map_or: {}", err.clone().map_or(-1, |v| v + 1));
    println!("iter: {:?}", ok.iter().chain(err.iter()).collect::<Vec<_>>());
    let nested: Result<Option<i32>, String> = Ok(Some(5));
    println!("transpose: {:?}", nested.transpose());
    let inputs = ["1", "2", "3"];
    let total: Result<i32, ParseIntError> = inputs.iter().map(|s| s.parse::<i32>()).sum();
    println!("sum of results: {:?}", total);
    let with_bad: Result<i32, _> = ["1", "oops"].iter().map(|s| s.parse::<i32>()).sum();
    println!("sum with bad: {:?}", with_bad.map_err(|e| e.to_string()));
    let product: Result<i64, ParseIntError> = ["2", "3", "7"].iter().map(|s| s.parse::<i64>()).product();
    println!("product: {:?}", product);
}
