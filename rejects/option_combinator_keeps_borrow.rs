// error: cannot borrow 'v' while 'v' is immutably borrowed
fn main() { let mut v = vec![1, 2]; let r: Result<&i32, ()> = v.first().ok_or(()); v.push(1); println!("{:?}", r); }
