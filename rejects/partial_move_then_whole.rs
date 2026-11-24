// error: use of partially moved value 'p'
#[derive(Debug)]
struct P { name: String }
fn main() {
    println!("start");
    let p = P { name: String::new() };
    let n = p.name;
    println!("{} {:?}", n, p);
}
