// error: while 'v' is mutably borrowed
fn main() {
    println!("start");
    let mut v = vec![1, 2];
    let first = v.first_mut().unwrap();
    let n = v.len();
    *first = n as i32;
}
