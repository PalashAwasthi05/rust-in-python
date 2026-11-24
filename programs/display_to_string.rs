use std::fmt;
struct P { x: i32, y: i32 }
impl fmt::Display for P {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "({}, {})", self.x, self.y) }
}
fn main() { let p = P { x: 1, y: 2 }; println!("{p}"); let s = p.to_string(); println!("{}", s.len()); }
