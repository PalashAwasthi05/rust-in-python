use std::rc::{Rc, Weak};
use std::cell::{RefCell, Cell};
use std::collections::HashMap;

#[derive(Debug)]
struct Node { value: i32, children: RefCell<Vec<Rc<Node>>>, parent: RefCell<Weak<Node>> }

struct Counter { hits: Cell<u32> }
impl Counter { fn hit(&self) { self.hits.set(self.hits.get() + 1); } }

fn main() {
    let shared = Rc::new(RefCell::new(vec![1]));
    let other = Rc::clone(&shared);
    other.borrow_mut().push(2);
    shared.borrow_mut().push(3);
    println!("{:?} {} {}", shared.borrow(), Rc::strong_count(&shared), shared.borrow().len());
    let leaf = Rc::new(Node { value: 3, children: RefCell::new(vec![]), parent: RefCell::new(Weak::new()) });
    println!("leaf parent = {:?}", leaf.parent.borrow().upgrade().map(|p| p.value));
    let branch = Rc::new(Node { value: 5, children: RefCell::new(vec![Rc::clone(&leaf)]), parent: RefCell::new(Weak::new()) });
    *leaf.parent.borrow_mut() = Rc::downgrade(&branch);
    println!("leaf parent = {:?}", leaf.parent.borrow().upgrade().map(|p| p.value));
    println!("{} {} {}", Rc::strong_count(&leaf), Rc::strong_count(&branch), Rc::weak_count(&branch));
    println!("{}", branch.children.borrow().iter().map(|c| c.value).sum::<i32>());
    let c = Counter { hits: Cell::new(0) };
    c.hit(); c.hit();
    println!("{}", c.hits.get());
    let cache: RefCell<HashMap<u64, u64>> = RefCell::new(HashMap::new());
    fn fib(n: u64, cache: &RefCell<HashMap<u64, u64>>) -> u64 {
        if n < 2 { return n; }
        if let Some(&v) = cache.borrow().get(&n) { return v; }
        let v = fib(n - 1, cache) + fib(n - 2, cache);
        cache.borrow_mut().insert(n, v);
        v
    }
    println!("{}", fib(50, &cache));
    let s = Rc::new(String::from("hello"));
    println!("{} {} {}", s.len(), s.to_uppercase(), *s == "hello");
    let list: Vec<Rc<RefCell<i32>>> = (0..3).map(|i| Rc::new(RefCell::new(i))).collect();
    for item in &list { *item.borrow_mut() *= 10; }
    println!("{:?}", list.iter().map(|x| *x.borrow()).collect::<Vec<_>>());
}
