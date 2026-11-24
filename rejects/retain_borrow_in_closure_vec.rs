// error: while 'v' is immutably borrowed
// known-gap: E0502: closure capture conflicts with the receiver's iterator borrow
fn main() {
    println!("start");
    let mut v = vec![1, 2];
    v.iter().for_each(|x| v.push(*x));
}
