// error: only traits defined in the current crate
use std::fmt;
impl fmt::Display for Vec<i32> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "v") }
}
fn main() {
    println!("start");
}
