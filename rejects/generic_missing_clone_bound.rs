// error: expected T, found &T
fn duplicate<T>(x: &T) -> (T, T) {
    (x.clone(), x.clone())
}
fn main() {
    println!("start");
    println!("{:?}", duplicate(&3));
}
