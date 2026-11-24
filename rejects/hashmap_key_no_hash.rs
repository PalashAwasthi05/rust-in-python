// error: does not implement Hash
// known-gap: E0277: HashMap::insert's K: Hash bound is not checked
use std::collections::HashMap;
#[derive(PartialEq, Eq)]
struct K(i32);
fn main() {
    println!("start");
    let mut m = HashMap::new();
    m.insert(K(1), 1);
    println!("{}", m.len());
}
