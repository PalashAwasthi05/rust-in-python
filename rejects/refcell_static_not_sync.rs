// error: cannot be shared between threads safely
use std::cell::RefCell;
static C: RefCell<i32> = RefCell::new(0);
fn main() {
    println!("start");
    println!("{}", C.borrow());
}
