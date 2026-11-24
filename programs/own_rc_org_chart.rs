use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::{Rc, Weak};

#[derive(Debug)]
struct Employee {
    name: String,
    manager: RefCell<Weak<Employee>>,
    reports: RefCell<Vec<Rc<Employee>>>,
    visits: Cell<u32>,
}

impl Drop for Employee {
    fn drop(&mut self) {
        println!("bye {}", self.name);
    }
}

fn hire(name: &str, boss: Option<&Rc<Employee>>) -> Rc<Employee> {
    let e = Rc::new(Employee {
        name: name.to_string(),
        manager: RefCell::new(Weak::new()),
        reports: RefCell::new(vec![]),
        visits: Cell::new(0),
    });
    if let Some(b) = boss {
        *e.manager.borrow_mut() = Rc::downgrade(b);
        b.reports.borrow_mut().push(Rc::clone(&e));
    }
    e
}

fn chain(e: &Rc<Employee>) -> Vec<String> {
    let mut out = vec![e.name.clone()];
    let mut cur = e.manager.borrow().upgrade();
    while let Some(m) = cur {
        m.visits.set(m.visits.get() + 1);
        out.push(m.name.clone());
        cur = m.manager.borrow().upgrade();
    }
    out
}

struct Observer {
    id: u32,
    log: Rc<RefCell<Vec<String>>>,
}

impl Observer {
    fn notify(&self, ev: &str) {
        self.log.borrow_mut().push(format!("{}:{}", self.id, ev));
    }
}

struct Subject {
    observers: Vec<Weak<Observer>>,
}

impl Subject {
    fn emit(&mut self, ev: &str) -> usize {
        self.observers.retain(|w| w.upgrade().is_some());
        for w in &self.observers {
            if let Some(o) = w.upgrade() {
                o.notify(ev);
            }
        }
        self.observers.len()
    }
}

fn main() {
    let ceo = hire("ceo", None);
    let cto = hire("cto", Some(&ceo));
    let dev = hire("dev", Some(&cto));
    let _ops = hire("ops", Some(&ceo));
    println!("{:?}", chain(&dev));
    println!("{:?}", chain(&dev));
    println!("ceo visits {} cto visits {}", ceo.visits.get(), cto.visits.get());
    println!("ceo reports {}", ceo.reports.borrow().len());
    println!("cto strong {} weak {}", Rc::strong_count(&cto), Rc::weak_count(&cto));
    drop(dev);
    println!("dev handle dropped; cto reports {}", cto.reports.borrow().len());
    cto.reports.borrow_mut().clear();
    println!("cleared");

    let log = Rc::new(RefCell::new(Vec::new()));
    let o1 = Rc::new(Observer { id: 1, log: Rc::clone(&log) });
    let mut subject = Subject { observers: vec![Rc::downgrade(&o1)] };
    {
        let o2 = Rc::new(Observer { id: 2, log: Rc::clone(&log) });
        subject.observers.push(Rc::downgrade(&o2));
        println!("live {}", subject.emit("a"));
    }
    println!("live {}", subject.emit("b"));
    println!("{:?}", log.borrow());
    println!("log strong {}", Rc::strong_count(&log));

    let registry: RefCell<HashMap<String, Rc<RefCell<i32>>>> = RefCell::new(HashMap::new());
    for k in ["x", "y", "x"] {
        let entry = Rc::clone(registry.borrow_mut().entry(k.to_string()).or_insert_with(|| Rc::new(RefCell::new(0))));
        *entry.borrow_mut() += 1;
    }
    let mut keys: Vec<_> = registry.borrow().keys().cloned().collect();
    keys.sort();
    for k in keys {
        println!("{} = {}", k, registry.borrow()[&k].borrow());
    }
    let counter = Rc::new(Cell::new(0));
    let handles: Vec<Rc<Cell<i32>>> = (0..3).map(|_| Rc::clone(&counter)).collect();
    for h in &handles {
        h.set(h.get() + 5);
    }
    println!("counter {} strong {}", counter.get(), Rc::strong_count(&counter));
    drop(handles);
    println!("strong {}", Rc::strong_count(&counter));
    println!("end");
}
