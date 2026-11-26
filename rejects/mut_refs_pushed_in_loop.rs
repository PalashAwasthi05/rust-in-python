// error: cannot borrow 'x' while 'x' is mutably borrowed
fn main() { let mut x = 0; let mut refs = vec![]; for _ in 0..2 { refs.push(&mut x); } }
