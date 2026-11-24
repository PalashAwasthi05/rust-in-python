// error: borrowed value 'b' does not live long enough
fn longest<'a>(a: &'a str, b: &'a str) -> &'a str { if a.len() > b.len() { a } else { b } }
fn main() {
    println!("start");
    let a = String::from("long");
    let r;
    {
        let b = String::from("x");
        r = longest(&a, &b);
    }
    println!("{}", r);
}
