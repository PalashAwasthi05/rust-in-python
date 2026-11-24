#[derive(Debug)]
enum Problem {
    Bad { code: u16, reason: String },
}

fn check(code: u16) -> Result<u16, Problem> {
    if code < 400 {
        Ok(code)
    } else {
        Err(Problem::Bad { code, reason: "client error".to_string() })
    }
}

fn main() {
    println!("{}", check(200).unwrap());
    println!("{}", check(301).unwrap());
    let parsed: Result<i32, _> = "12".parse::<i32>();
    println!("{}", parsed.unwrap());
    println!("{}", check(404).unwrap());
}
