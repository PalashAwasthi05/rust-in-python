// error: cannot borrow immutable place 'v' as mutable
fn main() {
    println!("start");
    let v = vec![1];
    let r = &mut v;
    r.push(2);
}
