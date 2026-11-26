// error: cannot borrow 'v' while 'v' is immutably borrowed
fn main() {
    let mut v = vec![1, 2, 3];
    let o = v.last();
    v.clear();
    println!("{}", o.unwrap());
}
