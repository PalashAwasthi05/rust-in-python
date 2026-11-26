// error: no two closures, even if identical, have the same type
fn main() { let a = 1; let mut h = || a; h = || a + 1; println!("{}", h()); }
