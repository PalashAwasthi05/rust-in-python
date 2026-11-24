// Chapter 15.4-15.6: Rc<T>, RefCell<T> interior mutability, and Weak<T> to avoid cycles.
use std::cell::RefCell;
use std::rc::{Rc, Weak};

#[derive(Debug)]
enum RcList {
    Cons(i32, Rc<RcList>),
    Nil,
}

#[derive(Debug)]
enum SharedList {
    Cons(Rc<RefCell<i32>>, Rc<SharedList>),
    Nil,
}

pub trait Messenger {
    fn send(&self, msg: &str);
}

pub struct LimitTracker<'a, T: Messenger> {
    messenger: &'a T,
    value: usize,
    max: usize,
}

impl<'a, T> LimitTracker<'a, T>
where
    T: Messenger,
{
    pub fn new(messenger: &'a T, max: usize) -> LimitTracker<'a, T> {
        LimitTracker { messenger, value: 0, max }
    }

    pub fn set_value(&mut self, value: usize) {
        self.value = value;
        let percentage_of_max = self.value as f64 / self.max as f64;
        if percentage_of_max >= 1.0 {
            self.messenger.send("Error: You are over your quota!");
        } else if percentage_of_max >= 0.9 {
            self.messenger.send("Urgent warning: You've used up over 90% of your quota!");
        } else if percentage_of_max >= 0.75 {
            self.messenger.send("Warning: You've used up over 75% of your quota!");
        }
    }
}

struct MockMessenger {
    sent_messages: RefCell<Vec<String>>,
}

impl MockMessenger {
    fn new() -> MockMessenger {
        MockMessenger { sent_messages: RefCell::new(vec![]) }
    }
}

impl Messenger for MockMessenger {
    fn send(&self, message: &str) {
        self.sent_messages.borrow_mut().push(String::from(message));
    }
}

#[derive(Debug)]
enum CycleList {
    Cons(i32, RefCell<Rc<CycleList>>),
    Nil,
}

impl CycleList {
    fn tail(&self) -> Option<&RefCell<Rc<CycleList>>> {
        match self {
            CycleList::Cons(_, item) => Some(item),
            CycleList::Nil => None,
        }
    }
}

#[derive(Debug)]
struct Node {
    value: i32,
    parent: RefCell<Weak<Node>>,
    children: RefCell<Vec<Rc<Node>>>,
}

impl Drop for Node {
    fn drop(&mut self) {
        println!("dropping node {}", self.value);
    }
}

fn main() {
    use RcList::{Cons, Nil};
    let a = Rc::new(Cons(5, Rc::new(Cons(10, Rc::new(Nil)))));
    println!("count after creating a = {}", Rc::strong_count(&a));
    let b = Cons(3, Rc::clone(&a));
    println!("count after creating b = {}", Rc::strong_count(&a));
    {
        let c = Cons(4, Rc::clone(&a));
        println!("count after creating c = {}", Rc::strong_count(&a));
        if let Cons(head, _) = &c {
            println!("c starts with {head}");
        }
    }
    println!("count after c goes out of scope = {}", Rc::strong_count(&a));
    println!("b = {:?}", b);

    let value = Rc::new(RefCell::new(5));
    let shared_a = Rc::new(SharedList::Cons(Rc::clone(&value), Rc::new(SharedList::Nil)));
    let shared_b = SharedList::Cons(Rc::new(RefCell::new(3)), Rc::clone(&shared_a));
    let shared_c = SharedList::Cons(Rc::new(RefCell::new(4)), Rc::clone(&shared_a));
    *value.borrow_mut() += 10;
    println!("a after = {shared_a:?}");
    println!("b after = {shared_b:?}");
    println!("c after = {shared_c:?}");

    let mock_messenger = MockMessenger::new();
    let mut limit_tracker = LimitTracker::new(&mock_messenger, 100);
    for v in [10, 80, 95, 120] {
        limit_tracker.set_value(v);
    }
    println!("value {} max {}", limit_tracker.value, limit_tracker.max);
    println!("messages sent: {}", mock_messenger.sent_messages.borrow().len());
    for m in mock_messenger.sent_messages.borrow().iter() {
        println!("  {m}");
    }

    let cell = RefCell::new(vec![1]);
    {
        let r1 = cell.borrow();
        let r2 = cell.borrow();
        println!("two shared borrows: {:?} {:?} try_borrow_mut ok? {}", r1, r2, cell.try_borrow_mut().is_ok());
    }
    cell.borrow_mut().push(2);
    println!("after mutation {:?} try_borrow_mut ok? {}", cell.borrow(), cell.try_borrow_mut().is_ok());

    let a = Rc::new(CycleList::Cons(5, RefCell::new(Rc::new(CycleList::Nil))));
    println!("a initial rc count = {}", Rc::strong_count(&a));
    println!("a next item = {:?}", a.tail());
    let b = Rc::new(CycleList::Cons(10, RefCell::new(Rc::clone(&a))));
    println!("a rc count after b creation = {}", Rc::strong_count(&a));
    println!("b initial rc count = {}", Rc::strong_count(&b));
    println!("b next item = {:?}", b.tail().map(|t| Rc::strong_count(&t.borrow())));
    if let Some(link) = a.tail() {
        *link.borrow_mut() = Rc::clone(&b);
    }
    println!("b rc count after changing a = {}", Rc::strong_count(&b));
    println!("a rc count after changing a = {}", Rc::strong_count(&a));

    let leaf = Rc::new(Node {
        value: 3,
        parent: RefCell::new(Weak::new()),
        children: RefCell::new(vec![]),
    });
    println!("leaf parent = {:?}", leaf.parent.borrow().upgrade().map(|p| p.value));
    println!("leaf strong = {}, weak = {}", Rc::strong_count(&leaf), Rc::weak_count(&leaf));
    {
        let branch = Rc::new(Node {
            value: 5,
            parent: RefCell::new(Weak::new()),
            children: RefCell::new(vec![Rc::clone(&leaf)]),
        });
        *leaf.parent.borrow_mut() = Rc::downgrade(&branch);
        println!("branch strong = {}, weak = {}", Rc::strong_count(&branch), Rc::weak_count(&branch));
        println!("leaf strong = {}, weak = {}", Rc::strong_count(&leaf), Rc::weak_count(&leaf));
        println!("leaf parent = {:?}", leaf.parent.borrow().upgrade().map(|p| p.value));
        let child_values: Vec<i32> = branch.children.borrow().iter().map(|c| c.value).collect();
        println!("branch children = {:?}", child_values);
    }
    println!("leaf parent = {:?}", leaf.parent.borrow().upgrade().map(|p| p.value));
    println!("leaf strong = {}, weak = {}", Rc::strong_count(&leaf), Rc::weak_count(&leaf));

    let shared = Rc::new(RefCell::new(String::from("log:")));
    let writers: Vec<Rc<RefCell<String>>> = (0..3).map(|_| Rc::clone(&shared)).collect();
    for (i, w) in writers.iter().enumerate() {
        w.borrow_mut().push_str(&format!(" w{i}"));
    }
    println!("{} (strong {})", shared.borrow(), Rc::strong_count(&shared));
    drop(writers);
    println!("strong after drop {}", Rc::strong_count(&shared));
    let inner = Rc::try_unwrap(shared).map(|c| c.into_inner());
    println!("{:?}", inner);
    println!("end of main");
}
