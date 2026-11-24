use std::rc::Rc;
struct Noisy { name: String }
impl Drop for Noisy { fn drop(&mut self) { println!("drop {}", self.name); } }
struct Pair { a: Noisy, b: Noisy }
fn make(n: &str) -> Noisy { Noisy { name: n.to_string() } }
fn consume(n: Noisy) { println!("consuming {}", n.name); }
fn main() {
    let _x = make("x");
    {
        let _inner = make("inner");
        println!("in block");
    }
    let y = make("y");
    consume(y);
    let p = Pair { a: make("a"), b: make("b") };
    let v = vec![make("v1"), make("v2")];
    let moved = v;
    println!("len {}", moved.len());
    std::mem::drop(p);
    let mut r = make("r1");
    r = make("r2");
    println!("reassigned {}", r.name);
    let shared = Rc::new(make("rc"));
    {
        let s2 = Rc::clone(&shared);
        println!("count {}", Rc::strong_count(&s2));
    }
    println!("count {}", Rc::strong_count(&shared));
    let boxed: Box<Noisy> = Box::new(make("boxed"));
    let opt = Some(make("opt"));
    if let Some(n) = opt { println!("got {}", n.name); }
    let _ = make("discarded");
    make("temp");
    println!("end {}", boxed.name);
}
