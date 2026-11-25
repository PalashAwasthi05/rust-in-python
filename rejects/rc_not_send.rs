// error: cannot be sent between threads safely
use std::rc::Rc;
fn main() {
    println!("start");
    let r = Rc::new(1);
    let h = std::thread::spawn(move || println!("{}", r));
    h.join().unwrap();
}
