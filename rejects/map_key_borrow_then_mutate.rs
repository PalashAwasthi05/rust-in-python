// error: cannot borrow 'k' while 'k' is immutably borrowed
use std::collections::HashMap;
fn main() {
    let mut m: HashMap<&str, i32> = HashMap::new();
    let mut k = String::from("a");
    m.insert(&k, 1);
    k.push('x');
    println!("{:?}", m);
}
