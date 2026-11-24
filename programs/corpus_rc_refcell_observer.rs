use std::cell::RefCell;
use std::rc::{Rc, Weak};

trait Observer {
    fn notify(&self, topic: &str, value: i32);
    fn name(&self) -> String;
}

struct Logger { lines: RefCell<Vec<String>> }
struct Totaler { total: RefCell<i32>, label: String }

impl Observer for Logger {
    fn notify(&self, topic: &str, value: i32) {
        self.lines.borrow_mut().push(format!("{}={}", topic, value));
    }
    fn name(&self) -> String { "logger".into() }
}

impl Observer for Totaler {
    fn notify(&self, _topic: &str, value: i32) {
        *self.total.borrow_mut() += value;
    }
    fn name(&self) -> String { self.label.clone() }
}

#[derive(Default)]
struct Subject { observers: Vec<Weak<dyn Observer>> }

impl Subject {
    fn subscribe(&mut self, o: &Rc<dyn Observer>) { self.observers.push(Rc::downgrade(o)); }
    fn publish(&mut self, topic: &str, value: i32) {
        self.observers.retain(|w| w.upgrade().is_some());
        for w in &self.observers {
            if let Some(o) = w.upgrade() { o.notify(topic, value); }
        }
    }
}

#[derive(Debug)]
struct TreeNode { val: i32, children: RefCell<Vec<Rc<TreeNode>>>, parent: RefCell<Weak<TreeNode>> }

fn main() {
    let logger = Rc::new(Logger { lines: RefCell::new(vec![]) });
    let logger_dyn: Rc<dyn Observer> = logger.clone();
    let mut subject = Subject::default();
    subject.subscribe(&logger_dyn);
    {
        let temp: Rc<dyn Observer> = Rc::new(Totaler { total: RefCell::new(0), label: "temp".into() });
        subject.subscribe(&temp);
        subject.publish("a", 5);
        println!("{} observers while temp alive, temp is {}", subject.observers.len(), temp.name());
    }
    let tot = Rc::new(Totaler { total: RefCell::new(100), label: "main".into() });
    let tot_dyn: Rc<dyn Observer> = tot.clone();
    subject.subscribe(&tot_dyn);
    subject.publish("b", 7);
    subject.publish("c", -2);
    println!("{} observers after temp dropped", subject.observers.len());
    println!("log: {:?}", logger.lines.borrow());
    println!("{} total: {}", tot.name(), tot.total.borrow());
    println!("logger strong={} weak={}", Rc::strong_count(&logger), Rc::weak_count(&logger));

    let leaf = Rc::new(TreeNode { val: 3, children: RefCell::new(vec![]), parent: RefCell::new(Weak::new()) });
    let branch = Rc::new(TreeNode { val: 5, children: RefCell::new(vec![leaf.clone()]), parent: RefCell::new(Weak::new()) });
    *leaf.parent.borrow_mut() = Rc::downgrade(&branch);
    println!("leaf parent = {:?}", leaf.parent.borrow().upgrade().map(|p| p.val));
    branch.children.borrow_mut().push(Rc::new(TreeNode { val: 9, children: RefCell::new(vec![]), parent: RefCell::new(Rc::downgrade(&branch)) }));
    let sum: i32 = branch.children.borrow().iter().map(|c| c.val).sum();
    println!("children sum = {}, leaf strong = {}", sum, Rc::strong_count(&leaf));
    drop(branch);
    println!("after drop, leaf parent = {:?}", leaf.parent.borrow().upgrade().map(|p| p.val));
}
