// error: cannot be shared between threads safely
// known-gap: E0277: Sync is not required of static types
use std::cell::RefCell;
static C: RefCell<i32> = RefCell::new(0);
fn main() {
    println!("start");
    println!("{}", C.borrow());
}
