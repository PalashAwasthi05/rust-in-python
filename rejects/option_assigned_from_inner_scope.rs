// error: 'v' does not live long enough
fn main() { let o; { let v = vec![1]; o = v.first(); } println!("{:?}", o); }
