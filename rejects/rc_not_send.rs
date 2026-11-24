// error: cannot be sent between threads safely
// known-gap: E0277: Send is not checked; currently rejected for an unrelated reason (thread join().unwrap() lacks Debug for Box<dyn Any>)
use std::rc::Rc;
fn main() {
    println!("start");
    let r = Rc::new(1);
    let h = std::thread::spawn(move || println!("{}", r));
    h.join().unwrap();
}
