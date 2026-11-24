// error: while 'v' is mutably borrowed
// known-gap: E0502: the borrow returned by first_mut().unwrap() is not traced
fn main() {
    println!("start");
    let mut v = vec![1, 2];
    let first = v.first_mut().unwrap();
    let n = v.len();
    *first = n as i32;
}
