// error: cannot borrow 'v' while 'v' is mutably borrowed
fn main() {
    println!("start");
    let mut v = vec![1];
    let a = &mut v[0];
    let b = &mut v[0];
    *a += 1;
    *b += 1;
}
