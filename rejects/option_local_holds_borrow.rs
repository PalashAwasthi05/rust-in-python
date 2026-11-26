// error: cannot borrow 'v' while 'v' is mutably borrowed
fn main() {
    let mut v = vec![1, 2, 3];
    let o = v.first_mut();
    v.push(4);
    if let Some(x) = o { *x += 1; }
    println!("{:?}", v);
}
