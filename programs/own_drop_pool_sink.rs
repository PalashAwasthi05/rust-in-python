use std::cell::RefCell;
use std::rc::Rc;

struct Logger {
    name: String,
    sink: Rc<RefCell<Vec<String>>>,
}

impl Drop for Logger {
    fn drop(&mut self) {
        self.sink.borrow_mut().push(format!("drop {}", self.name));
    }
}

fn mk(name: &str, sink: &Rc<RefCell<Vec<String>>>) -> Logger {
    sink.borrow_mut().push(format!("new {}", name));
    Logger { name: name.to_string(), sink: Rc::clone(sink) }
}

struct Pool {
    items: Vec<Logger>,
}

impl Pool {
    fn take_first(&mut self) -> Option<Logger> {
        if self.items.is_empty() { None } else { Some(self.items.remove(0)) }
    }
    fn replace_last(&mut self, l: Logger) -> Option<Logger> {
        let old = self.items.pop();
        self.items.push(l);
        old
    }
}

fn main() {
    let sink = Rc::new(RefCell::new(Vec::new()));
    {
        let mut pool = Pool { items: vec![mk("a", &sink), mk("b", &sink), mk("c", &sink)] };
        let first = pool.take_first();
        sink.borrow_mut().push(format!("took {}", first.as_ref().map(|l| l.name.as_str()).unwrap_or("-")));
        let old = pool.replace_last(mk("d", &sink));
        drop(old);
        let names: Vec<String> = pool.items.iter().map(|l| l.name.clone()).collect();
        sink.borrow_mut().push(format!("pool {:?}", names));
        let kept: Vec<Logger> = pool.items.drain(..).filter(|l| l.name != "b").collect();
        sink.borrow_mut().push(format!("kept {}", kept.len()));
        let mut sorted = vec![mk("z", &sink), mk("y", &sink)];
        sorted.sort_by(|a, b| a.name.cmp(&b.name));
        sink.borrow_mut().push(format!("sorted first {}", sorted[0].name));
        let swapped = std::mem::replace(&mut sorted[0], mk("x", &sink));
        sink.borrow_mut().push(format!("swapped out {}", swapped.name));
    }
    {
        let make = |n: &str| mk(n, &sink);
        let v: Vec<Logger> = ["p", "q"].iter().map(|n| make(n)).collect();
        let total: usize = v.iter().map(|l| l.name.len()).sum();
        sink.borrow_mut().push(format!("total {}", total));
        let found = v.into_iter().find(|l| l.name == "p");
        sink.borrow_mut().push(format!("found {}", found.is_some()));
    }
    {
        let opt = Some(mk("o", &sink));
        let mapped = opt.map(|l| l.name.len());
        sink.borrow_mut().push(format!("mapped {:?}", mapped));
        let r: Result<Logger, String> = Ok(mk("r", &sink));
        if let Ok(ref l) = r {
            sink.borrow_mut().push(format!("ok {}", l.name));
        }
        let e: Result<Logger, Logger> = Err(mk("e", &sink));
        let v = e.unwrap_or_else(|err| {
            let n = format!("{}2", err.name);
            mk(&n, &sink)
        });
        sink.borrow_mut().push(format!("got {}", v.name));
    }
    for line in sink.borrow().iter() {
        println!("{}", line);
    }
    println!("refs {}", Rc::strong_count(&sink));
}
