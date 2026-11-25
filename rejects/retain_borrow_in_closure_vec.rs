// error: while 'v' is immutably borrowed
fn main() {
    println!("start");
    let mut v = vec![1, 2];
    v.iter().for_each(|x| v.push(*x));
}
