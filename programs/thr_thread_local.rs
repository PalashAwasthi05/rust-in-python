use std::cell::{Cell, RefCell};
use std::thread;

thread_local! {
    static COUNTER: Cell<u32> = Cell::new(0);
    static LOG: RefCell<Vec<String>> = RefCell::new(Vec::new());
}

thread_local!(static DEPTH: Cell<usize> = const { Cell::new(1) });

fn bump() -> u32 {
    COUNTER.with(|c| {
        c.set(c.get() + 1);
        c.get()
    })
}

fn main() {
    bump();
    bump();
    println!("main counter {}", COUNTER.with(|c| c.get()));
    let handles: Vec<_> = (0..3)
        .map(|i| {
            thread::spawn(move || {
                for _ in 0..=i {
                    bump();
                }
                LOG.with(|log| log.borrow_mut().push(format!("thread {}", i)));
                let n = LOG.with(|log| log.borrow().len());
                (COUNTER.with(|c| c.get()), n)
            })
        })
        .collect();
    for h in handles {
        let (count, logged) = h.join().unwrap();
        println!("thread saw counter {} log {}", count, logged);
    }
    println!("main counter still {}", COUNTER.with(Cell::get));
    LOG.with(|log| log.borrow_mut().push(String::from("main")));
    LOG.with_borrow(|log| println!("main log {:?}", log));
    LOG.with_borrow_mut(|log| log.push(String::from("again")));
    println!("log len {}", LOG.with_borrow(|log| log.len()));
    COUNTER.set(42);
    println!("after set {}", COUNTER.get());
    println!("taken {} now {}", COUNTER.take(), COUNTER.get());
    DEPTH.with(|d| d.set(d.get() * 10));
    let other = thread::spawn(|| DEPTH.get()).join().unwrap();
    println!("depth main {} other {}", DEPTH.get(), other);
}
