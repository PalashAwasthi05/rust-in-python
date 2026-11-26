use std::collections::HashMap;
use std::borrow::Cow;
struct W<T>(T);
impl From<i32> for W<i64> { fn from(x: i32) -> Self { W(x as i64) } }
fn main() {
    let m = HashMap::from([(1, 2)]); println!("{:?}", m);
    let w = W::from(3); println!("{}", w.0);
    let c: Cow<str> = Cow::from("x"); println!("{}", c);
}
