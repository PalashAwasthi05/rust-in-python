use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

struct Queue {
    items: Mutex<VecDeque<u32>>,
    available: Condvar,
}

impl Queue {
    fn push(&self, value: u32) {
        self.items.lock().unwrap().push_back(value);
        self.available.notify_one();
    }

    fn pop(&self) -> u32 {
        let mut items = self.items.lock().unwrap();
        loop {
            if let Some(v) = items.pop_front() {
                return v;
            }
            items = self.available.wait(items).unwrap();
        }
    }
}

fn main() {
    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let pair2 = Arc::clone(&pair);
    let worker = thread::spawn(move || {
        let (lock, cvar) = &*pair2;
        let mut started = lock.lock().unwrap();
        *started = true;
        cvar.notify_one();
        println!("worker signalled");
    });
    let (lock, cvar) = &*pair;
    let mut started = lock.lock().unwrap();
    while !*started {
        started = cvar.wait(started).unwrap();
    }
    println!("main saw started = {}", *started);
    drop(started);
    worker.join().unwrap();

    let state = Arc::new((Mutex::new(Vec::<u32>::new()), Condvar::new()));
    let mut handles = Vec::new();
    for i in 0..4 {
        let state = Arc::clone(&state);
        handles.push(thread::spawn(move || {
            let (items, cv) = &*state;
            items.lock().unwrap().push(i * 3);
            cv.notify_all();
        }));
    }
    {
        let (items, cv) = &*state;
        let guard = cv.wait_while(items.lock().unwrap(), |v| v.len() < 4).unwrap();
        let mut sorted = guard.clone();
        sorted.sort();
        println!("collected {:?}", sorted);
    }
    for h in handles {
        h.join().unwrap();
    }

    let queue = Arc::new(Queue { items: Mutex::new(VecDeque::new()), available: Condvar::new() });
    let consumer = {
        let queue = Arc::clone(&queue);
        thread::spawn(move || {
            let mut sum = 0;
            loop {
                let v = queue.pop();
                if v == 0 {
                    break;
                }
                sum += v;
            }
            sum
        })
    };
    for v in [4, 8, 15, 16, 23, 42, 0] {
        queue.push(v);
    }
    println!("consumer summed {}", consumer.join().unwrap());
}
