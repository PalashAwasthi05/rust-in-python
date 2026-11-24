// Doubly linked list with Rc<RefCell<>> and Weak back links.
use std::cell::RefCell;
use std::rc::{Rc, Weak};

type Link = Option<Rc<RefCell<Node>>>;

struct Node {
    value: i32,
    next: Link,
    prev: Option<Weak<RefCell<Node>>>,
}

struct List {
    head: Link,
    tail: Link,
    len: usize,
}

impl List {
    fn new() -> Self {
        List { head: None, tail: None, len: 0 }
    }

    fn push_back(&mut self, value: i32) {
        let node = Rc::new(RefCell::new(Node { value, next: None, prev: None }));
        match self.tail.take() {
            Some(old) => {
                node.borrow_mut().prev = Some(Rc::downgrade(&old));
                old.borrow_mut().next = Some(Rc::clone(&node));
                self.tail = Some(node);
            }
            None => {
                self.head = Some(node.clone());
                self.tail = Some(node);
            }
        }
        self.len += 1;
    }

    fn pop_front(&mut self) -> Option<i32> {
        self.head.take().map(|old| {
            match old.borrow_mut().next.take() {
                Some(next) => {
                    next.borrow_mut().prev = None;
                    self.head = Some(next);
                }
                None => {
                    self.tail = None;
                }
            }
            self.len -= 1;
            let v = old.borrow().value;
            v
        })
    }

    fn to_vec(&self) -> Vec<i32> {
        let mut out = Vec::new();
        let mut cur = self.head.clone();
        while let Some(n) = cur {
            out.push(n.borrow().value);
            cur = n.borrow().next.clone();
        }
        out
    }

    fn to_vec_rev(&self) -> Vec<i32> {
        let mut out = Vec::new();
        let mut cur = self.tail.clone();
        while let Some(n) = cur {
            out.push(n.borrow().value);
            cur = n.borrow().prev.as_ref().and_then(|w| w.upgrade());
        }
        out
    }

    fn map_in_place(&self, f: impl Fn(i32) -> i32) {
        let mut cur = self.head.clone();
        while let Some(n) = cur {
            let v = n.borrow().value;
            n.borrow_mut().value = f(v);
            cur = n.borrow().next.clone();
        }
    }
}

fn main() {
    let mut list = List::new();
    for i in 1..=5 {
        list.push_back(i * 10);
    }
    println!("{:?} len={}", list.to_vec(), list.len);
    println!("{:?}", list.to_vec_rev());
    list.map_in_place(|v| v + 1);
    println!("{:?}", list.to_vec());
    println!("{:?} {:?}", list.pop_front(), list.pop_front());
    println!("{:?} len={}", list.to_vec(), list.len);
    println!("{:?}", list.to_vec_rev());
    if let Some(h) = &list.head {
        println!("head strong={} weak={}", Rc::strong_count(h), Rc::weak_count(h));
    }
    while let Some(v) = list.pop_front() {
        println!("popped {}", v);
    }
    println!("empty: {} {}", list.head.is_none(), list.tail.is_none());

    let shared = Rc::new(RefCell::new(vec![1, 2]));
    let a = Rc::clone(&shared);
    let b = Rc::clone(&shared);
    a.borrow_mut().push(3);
    b.borrow_mut().push(4);
    {
        let r1 = shared.borrow();
        let r2 = shared.borrow();
        println!("{:?} {:?} {}", *r1, *r2, shared.try_borrow_mut().is_err());
    }
    println!("{}", shared.try_borrow_mut().is_ok());
    println!("count={}", Rc::strong_count(&shared));
}
