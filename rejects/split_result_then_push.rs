// error: cannot borrow 'v' while 'v' is mutably borrowed
fn halves(v: &mut [i32]) -> (&mut [i32], &mut [i32]) { let n = v.len() / 2; v.split_at_mut(n) }
fn main() { let mut v = vec![1, 2, 3, 4]; let (x, y) = halves(&mut v); v.push(1); x[0] = y[0]; }
