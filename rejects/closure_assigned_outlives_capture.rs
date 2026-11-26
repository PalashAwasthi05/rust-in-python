// error: 's' does not live long enough
fn main() { let f; { let s = String::from("a"); f = || s.len(); } f(); }
