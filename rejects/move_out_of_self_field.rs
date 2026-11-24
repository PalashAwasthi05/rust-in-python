// error: cannot move out of borrowed place
struct P { name: String }
impl P { fn name(&self) -> String { self.name } }
fn main() {
    println!("start");
    println!("{}", P { name: String::new() }.name());
}
