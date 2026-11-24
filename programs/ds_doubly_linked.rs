// Doubly linked list using Rc<RefCell<Node>> for forward links and Weak for back links.
use std::cell::RefCell;
use std::rc::{Rc, Weak};

type Strong<T> = Rc<RefCell<DNode<T>>>;

struct DNode<T> {
    value: T,
    next: Option<Strong<T>>,
    prev: Option<Weak<RefCell<DNode<T>>>>,
}

struct Deque<T> {
    head: Option<Strong<T>>,
    tail: Option<Strong<T>>,
    len: usize,
}

impl<T: Clone + std::fmt::Debug> Deque<T> {
    fn new() -> Self {
        Deque { head: None, tail: None, len: 0 }
    }

    fn push_front(&mut self, value: T) {
        let node = Rc::new(RefCell::new(DNode { value, next: None, prev: None }));
        match self.head.take() {
            Some(old) => {
                old.borrow_mut().prev = Some(Rc::downgrade(&node));
                node.borrow_mut().next = Some(old);
                self.head = Some(node);
            }
            None => {
                self.tail = Some(Rc::clone(&node));
                self.head = Some(node);
            }
        }
        self.len += 1;
    }

    fn push_back(&mut self, value: T) {
        let node = Rc::new(RefCell::new(DNode { value, next: None, prev: None }));
        match self.tail.take() {
            Some(old) => {
                node.borrow_mut().prev = Some(Rc::downgrade(&old));
                old.borrow_mut().next = Some(Rc::clone(&node));
                self.tail = Some(node);
            }
            None => {
                self.head = Some(Rc::clone(&node));
                self.tail = Some(node);
            }
        }
        self.len += 1;
    }

    fn pop_front(&mut self) -> Option<T> {
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
            let v = old.borrow().value.clone();
            v
        })
    }

    fn pop_back(&mut self) -> Option<T> {
        self.tail.take().map(|old| {
            let prev = old.borrow_mut().prev.take().and_then(|w| w.upgrade());
            match prev {
                Some(p) => {
                    p.borrow_mut().next = None;
                    self.tail = Some(p);
                }
                None => {
                    self.head = None;
                }
            }
            self.len -= 1;
            match Rc::try_unwrap(old) {
                Ok(cell) => cell.into_inner().value,
                Err(_) => panic!("node still shared"),
            }
        })
    }

    fn peek_front(&self) -> Option<T> {
        self.head.as_ref().map(|n| n.borrow().value.clone())
    }

    fn peek_back(&self) -> Option<T> {
        self.tail.as_ref().map(|n| n.borrow().value.clone())
    }

    fn to_vec(&self) -> Vec<T> {
        let mut out = Vec::new();
        let mut cur = self.head.clone();
        while let Some(node) = cur {
            out.push(node.borrow().value.clone());
            cur = node.borrow().next.clone();
        }
        out
    }

    fn to_vec_rev(&self) -> Vec<T> {
        let mut out = Vec::new();
        let mut cur = self.tail.clone();
        while let Some(node) = cur {
            out.push(node.borrow().value.clone());
            cur = node.borrow().prev.as_ref().and_then(|w| w.upgrade());
        }
        out
    }
}

impl<T> Drop for Deque<T> {
    fn drop(&mut self) {
        let mut cur = self.head.take();
        self.tail = None;
        while let Some(node) = cur {
            cur = node.borrow_mut().next.take();
        }
    }
}

struct Tracked(&'static str);

impl Drop for Tracked {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

fn main() {
    let mut d: Deque<i32> = Deque::new();
    for i in 1..=4 {
        d.push_back(i);
    }
    d.push_front(0);
    d.push_front(-1);
    println!("forward:  {:?}", d.to_vec());
    println!("backward: {:?}", d.to_vec_rev());
    println!("len {} front {:?} back {:?}", d.len, d.peek_front(), d.peek_back());

    println!("pop_back {:?}", d.pop_back());
    println!("pop_front {:?}", d.pop_front());
    println!("forward:  {:?}", d.to_vec());
    println!("backward: {:?}", d.to_vec_rev());

    if let Some(head) = &d.head {
        let next = head.borrow().next.clone().unwrap();
        println!("head strong={} weak={}", Rc::strong_count(head), Rc::weak_count(head));
        println!("second strong={} weak={}", Rc::strong_count(&next), Rc::weak_count(&next));
        next.borrow_mut().value *= 100;
    }
    println!("after edit: {:?}", d.to_vec());

    while let Some(v) = d.pop_back() {
        print!("{} ", v);
    }
    println!("| len {}", d.len);
    println!("empty peeks: {:?} {:?}", d.peek_front(), d.peek_back());

    let mut names: Deque<String> = Deque::new();
    names.push_back("b".to_string());
    names.push_front("a".to_string());
    names.push_back("c".to_string());
    println!("names {:?} rev {:?}", names.to_vec(), names.to_vec_rev());

    // Weak back-pointer stops dangling when the node is gone
    let weak_probe;
    {
        let mut tmp: Deque<u8> = Deque::new();
        tmp.push_back(1);
        tmp.push_back(2);
        weak_probe = Rc::downgrade(tmp.head.as_ref().unwrap());
        println!("probe alive: {:?}", weak_probe.upgrade().map(|n| n.borrow().value));
    }
    println!("probe after scope: {:?}", weak_probe.upgrade().map(|n| n.borrow().value));

    let a = Rc::new(RefCell::new(Vec::<Tracked>::new()));
    a.borrow_mut().push(Tracked("first"));
    a.borrow_mut().push(Tracked("second"));
    let b = Rc::clone(&a);
    println!("shared count {}", Rc::strong_count(&a));
    drop(a);
    println!("after dropping a: {}", Rc::strong_count(&b));
    b.borrow_mut().remove(0);
    println!("end of main");
}
