// RAII guards that borrow mutably and restore state in Drop.
use std::cell::{Cell, RefCell};

struct Indent<'a> {
    level: &'a Cell<usize>,
}

impl<'a> Indent<'a> {
    fn new(level: &'a Cell<usize>) -> Self {
        level.set(level.get() + 1);
        Indent { level }
    }
}

impl Drop for Indent<'_> {
    fn drop(&mut self) {
        self.level.set(self.level.get() - 1);
    }
}

fn log(level: &Cell<usize>, msg: &str) {
    println!("{}{}", "  ".repeat(level.get()), msg);
}

struct Transaction<'a> {
    target: &'a mut Vec<i32>,
    saved_len: usize,
    committed: bool,
}

impl<'a> Transaction<'a> {
    fn begin(target: &'a mut Vec<i32>) -> Self {
        let saved_len = target.len();
        Transaction { target, saved_len, committed: false }
    }
    fn push(&mut self, v: i32) {
        self.target.push(v);
    }
    fn commit(mut self) {
        self.committed = true;
    }
}

impl Drop for Transaction<'_> {
    fn drop(&mut self) {
        if !self.committed {
            println!("rollback to {}", self.saved_len);
            self.target.truncate(self.saved_len);
        } else {
            println!("commit at {}", self.target.len());
        }
    }
}

struct Tracker<'a> {
    name: String,
    events: &'a RefCell<Vec<String>>,
}

impl Drop for Tracker<'_> {
    fn drop(&mut self) {
        self.events.borrow_mut().push(format!("end {}", self.name));
    }
}

fn traced<'a>(name: &str, events: &'a RefCell<Vec<String>>) -> Tracker<'a> {
    events.borrow_mut().push(format!("start {}", name));
    Tracker { name: name.to_string(), events }
}

fn visit(depth: u32, level: &Cell<usize>) {
    let _g = Indent::new(level);
    log(level, &format!("depth {}", depth));
    if depth < 2 {
        visit(depth + 1, level);
        visit(depth + 1, level);
    }
}

fn main() {
    let level = Cell::new(0);
    log(&level, "root");
    visit(0, &level);
    log(&level, "back at root");

    let mut data = vec![1, 2];
    {
        let mut tx = Transaction::begin(&mut data);
        tx.push(3);
        tx.push(4);
    }
    println!("{:?}", data);
    {
        let mut tx = Transaction::begin(&mut data);
        tx.push(5);
        tx.commit();
    }
    println!("{:?}", data);
    data.push(6);
    println!("{:?}", data);

    let events = RefCell::new(Vec::new());
    {
        let _outer = traced("outer", &events);
        {
            let _a = traced("a", &events);
            let _b = traced("b", &events);
        }
        let t = traced("c", &events);
        drop(t);
        let _d = traced("d", &events);
    }
    for e in events.borrow().iter() {
        println!("{}", e);
    }
}
