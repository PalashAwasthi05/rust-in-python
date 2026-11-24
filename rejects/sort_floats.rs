// error: does not implement Ord
// known-gap: E0277: method-level where-bounds on std methods (sort needs T: Ord) are not checked
fn main() {
    println!("start");
    let mut v = vec![1.0, 0.5];
    v.sort();
    println!("{:?}", v);
}
