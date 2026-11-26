// error: cannot borrow 'v' while 'v' is immutably borrowed
fn main() { let mut v = vec![1, 2]; let z = 0; let mut r = &z; r = &v[0]; v.push(3); println!("{:?}", r); }
