// error: cannot borrow 'v' while 'v' is immutably borrowed
fn main() {
    println!("start");
    let mut v = vec![1, 2, 3];
    let first = &v[0];
    v.push(4);
    println!("{}", first);
}
