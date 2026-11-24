// RAII guards with Drop: scope timers, transactions that roll back, lock-like guards.
use std::cell::{Cell, RefCell};
use std::ops::{Deref, DerefMut};

struct Indent<'a> {
    depth: &'a Cell<usize>,
    name: String,
}

impl<'a> Indent<'a> {
    fn enter(depth: &'a Cell<usize>, name: &str) -> Indent<'a> {
        println!("{}> {}", "  ".repeat(depth.get()), name);
        depth.set(depth.get() + 1);
        Indent { depth, name: name.to_string() }
    }
}

impl Drop for Indent<'_> {
    fn drop(&mut self) {
        self.depth.set(self.depth.get() - 1);
        println!("{}< {}", "  ".repeat(self.depth.get()), self.name);
    }
}

struct Transaction<'a> {
    log: &'a RefCell<Vec<String>>,
    pending: Vec<String>,
    committed: bool,
}

impl<'a> Transaction<'a> {
    fn begin(log: &'a RefCell<Vec<String>>) -> Self {
        Transaction { log, pending: Vec::new(), committed: false }
    }
    fn write(&mut self, s: &str) {
        self.pending.push(s.to_string());
    }
    fn commit(mut self) {
        self.log.borrow_mut().extend(self.pending.drain(..));
        self.committed = true;
    }
}

impl Drop for Transaction<'_> {
    fn drop(&mut self) {
        if self.committed {
            println!("transaction committed");
        } else {
            println!("transaction rolled back ({} pending writes dropped)", self.pending.len());
        }
    }
}

struct Mutexish<T> {
    value: RefCell<T>,
    locked: Cell<bool>,
}

struct Guard<'a, T> {
    owner: &'a Mutexish<T>,
    value: std::cell::RefMut<'a, T>,
}

impl<T> Mutexish<T> {
    fn new(v: T) -> Self {
        Mutexish { value: RefCell::new(v), locked: Cell::new(false) }
    }
    fn lock(&self) -> Option<Guard<'_, T>> {
        if self.locked.get() {
            return None;
        }
        self.locked.set(true);
        println!("locked");
        Some(Guard { owner: self, value: self.value.borrow_mut() })
    }
}

impl<'a, T> Deref for Guard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<'a, T> DerefMut for Guard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<'a, T> Drop for Guard<'a, T> {
    fn drop(&mut self) {
        self.owner.locked.set(false);
        println!("unlocked");
    }
}

struct Noisy(&'static str);
impl Drop for Noisy {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

struct Pair {
    _first: Noisy,
    _second: Noisy,
}

fn work(fail: bool, log: &RefCell<Vec<String>>) -> Result<(), String> {
    let mut tx = Transaction::begin(log);
    tx.write("a");
    tx.write("b");
    if fail {
        return Err("failed midway".to_string());
    }
    tx.write("c");
    tx.commit();
    Ok(())
}

fn main() {
    let depth = Cell::new(0);
    {
        let _outer = Indent::enter(&depth, "outer");
        {
            let _inner = Indent::enter(&depth, "inner");
            let _deeper = Indent::enter(&depth, "deeper");
            println!("{}working", "  ".repeat(depth.get()));
        }
        let early = Indent::enter(&depth, "early");
        drop(early);
        println!("depth now {}", depth.get());
    }

    let log = RefCell::new(Vec::new());
    println!("{:?}", work(false, &log));
    println!("{:?}", work(true, &log));
    println!("log = {:?}", log.borrow());

    let m = Mutexish::new(vec![1, 2, 3]);
    {
        let mut g = m.lock().unwrap();
        g.push(4);
        println!("second lock: {}", m.lock().is_none());
        println!("inside: {:?} len {}", *g, g.len());
    }
    println!("relock ok: {}", m.lock().map(|g| g.len()).unwrap_or(0));

    let _a = Noisy("a");
    let _b = Noisy("b");
    let p = Pair { _first: Noisy("pair.first"), _second: Noisy("pair.second") };
    let v = vec![Noisy("v0"), Noisy("v1")];
    let moved = p;
    let _ = Noisy("temporary");
    std::mem::drop(v);
    let boxed: Box<Noisy> = Box::new(Noisy("boxed"));
    let opt = Some(Noisy("opt"));
    if let Some(n) = opt {
        println!("took {}", n.0);
    }
    println!("end of main {}", boxed.0);
    let _keep = moved;
}
