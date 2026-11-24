fn f() -> Result<i32, Box<dyn std::error::Error>> { Err("zero".into()) }
fn main() { let s: String = "abc".into(); println!("{s} {}", f().is_err()); }
