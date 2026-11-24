// error: only traits defined in the current crate
// known-gap: E0117: orphan rule is not enforced
use std::fmt;
impl fmt::Display for Vec<i32> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "v") }
}
fn main() {
    println!("start");
}
