// error: cannot borrow 'v' while 'v' is mutably borrowed
fn main() {
    let mut v = vec![1, 2, 3];
    let (a, b) = v.split_at_mut(1);
    v.push(4);
    a[0] += b[0];
    println!("{:?}", v);
}
