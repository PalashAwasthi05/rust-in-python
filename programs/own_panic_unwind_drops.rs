use std::rc::Rc;
use std::cell::RefCell;

trait Plugin {
    fn run(&self) -> String;
}

struct Echo(String);
impl Plugin for Echo {
    fn run(&self) -> String { format!("echo {}", self.0) }
}
impl Drop for Echo {
    fn drop(&mut self) { println!("unload {}", self.0); }
}

struct Guard<T: std::fmt::Display>(T);
impl<T: std::fmt::Display> Drop for Guard<T> {
    fn drop(&mut self) { println!("guard {} released", self.0); }
}

fn risky(n: i32) -> i32 {
    let _g = Guard(format!("risky{}", n));
    let v: Vec<i32> = vec![1, 2, 3];
    v[n as usize]
}

fn main() {
    let plugins: Vec<Rc<dyn Plugin>> = vec![Rc::new(Echo("a".into())), Rc::new(Echo("b".into()))];
    let shared = Rc::clone(&plugins[1]);
    for p in &plugins {
        println!("{}", p.run());
    }
    drop(plugins);
    println!("plugins dropped, shared {}", shared.run());
    let boxed: Box<dyn Plugin> = Box::new(Echo("c".into()));
    let cell: RefCell<Option<Box<dyn Plugin>>> = RefCell::new(Some(boxed));
    let taken = cell.borrow_mut().take();
    println!("taken {}", taken.map(|p| p.run()).unwrap_or_default());
    println!("cell empty {}", cell.borrow().is_none());
    let _outer = Guard(1);
    let _second = Guard("two");
    println!("{}", risky(1));
    println!("{}", risky(5));
    println!("unreachable");
}
