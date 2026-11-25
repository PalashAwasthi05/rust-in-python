// error: does not implement Hash
use std::collections::HashMap;
#[derive(PartialEq, Eq)]
struct K(i32);
fn main() {
    println!("start");
    let mut m = HashMap::new();
    m.insert(K(1), 1);
    println!("{}", m.len());
}
