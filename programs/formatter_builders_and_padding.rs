use std::fmt;
struct P { x: i32, y: i32 }
impl fmt::Display for P { fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "({}, {})", self.x, self.y) } }
struct Name(String);
impl fmt::Display for Name { fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { f.pad(&self.0) } }
struct Secret { user: String, key: u32 }
impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { f.debug_struct("Secret").field("user", &self.user).finish_non_exhaustive() }
}
struct Pair(i32, String);
impl fmt::Debug for Pair {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { f.debug_tuple("Pair").field(&self.0).field(&self.1).finish() }
}
struct Bag(Vec<u8>);
impl fmt::Debug for Bag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { f.debug_list().entries(self.0.iter()).finish() }
}
struct Prec(f64);
impl fmt::Display for Prec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(p) = f.precision() { write!(f, "~{:.*}", p, self.0) } else { write!(f, "~{}", self.0) }
    }
}
fn main() {
    let p = P { x: 1, y: 2 };
    println!("[{:>12}] [{:<12}] [{}]", p, p, p);
    println!("[{:>8}] [{:-^9}] [{}]", Name("ann".into()), Name("bo".into()), Name("cy".into()));
    let s = Secret { user: "root".into(), key: 7 };
    println!("{:?}", s);
    println!("{:#?}", s);
    println!("{:?} {:?}", Pair(1, "a".into()), Bag(vec![1, 2]));
    println!("{:#?}", Pair(1, "a".into()));
    println!("{} {:.2}", Prec(3.14159), Prec(3.14159));
}
