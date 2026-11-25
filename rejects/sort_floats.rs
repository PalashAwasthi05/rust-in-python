// error: does not implement Ord
fn main() {
    println!("start");
    let mut v = vec![1.0, 0.5];
    v.sort();
    println!("{:?}", v);
}
