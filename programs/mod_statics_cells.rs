// Statics with interior mutability and pub(crate) functions in modules.
mod counter {
    use std::sync::atomic::{AtomicUsize, Ordering};

    static HITS: AtomicUsize = AtomicUsize::new(0);
    pub static NAME: &str = "counter";

    pub(crate) fn hit() -> usize {
        HITS.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub fn hits() -> usize {
        HITS.load(Ordering::SeqCst)
    }

    pub(self) fn reset_hidden() {
        HITS.store(0, Ordering::SeqCst);
    }

    pub fn reset() {
        reset_hidden();
    }
}

mod registry {
    use std::cell::RefCell;

    pub struct Log {
        entries: RefCell<Vec<String>>,
    }

    impl Log {
        pub const fn new() -> Log {
            Log { entries: RefCell::new(Vec::new()) }
        }

        pub fn record(&self, s: &str) {
            self.entries.borrow_mut().push(s.to_string());
        }

        pub fn dump(&self) -> Vec<String> {
            self.entries.borrow().clone()
        }
    }
}

fn main() {
    counter::hit();
    counter::hit();
    println!("{} {}", counter::NAME, counter::hit());
    println!("{}", counter::hits());
    counter::reset();
    println!("{}", counter::hits());
    let log = registry::Log::new();
    log.record("a");
    log.record("b");
    println!("{:?}", log.dump());
}
