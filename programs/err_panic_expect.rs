use std::collections::HashMap;

fn main() {
    let mut ages = HashMap::new();
    ages.insert("ann", 31);
    ages.insert("ben", 27);
    let ann = ages.get("ann").expect("ann should exist");
    println!("ann is {}", ann);
    let n: u8 = "250".parse().expect("valid u8");
    println!("n = {}", n);
    let m: u8 = "260".parse().expect("config value must fit in a u8");
    println!("m = {}", m);
}
