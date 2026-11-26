// error: 'b' does not live long enough
use std::collections::HashMap;
fn main() {
    let mut refs: Vec<&String> = Vec::new();
    let a = String::from("a");
    refs.push(&a);
    { let b = String::from("b"); refs.push(&b); }
    println!("{:?}", refs);
}
