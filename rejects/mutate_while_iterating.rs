// error: cannot borrow 'v' while 'v' is immutably borrowed
fn main() {
    println!("start");
    let mut v = vec![1, 2, 3];
    for x in &v {
        v.push(*x);
    }
}
