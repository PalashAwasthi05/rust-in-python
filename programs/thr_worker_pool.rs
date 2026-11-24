use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

type Job = Box<dyn FnOnce() -> u64 + Send + 'static>;

struct Worker {
    id: usize,
    handle: Option<thread::JoinHandle<usize>>,
}

struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<Sender<Job>>,
}

impl ThreadPool {
    fn new(size: usize, results: Sender<u64>) -> ThreadPool {
        assert!(size > 0);
        let (sender, receiver) = mpsc::channel::<Job>();
        let receiver: Arc<Mutex<Receiver<Job>>> = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            let receiver = Arc::clone(&receiver);
            let results = results.clone();
            let handle = thread::spawn(move || {
                let mut done = 0;
                loop {
                    let message = receiver.lock().unwrap().recv();
                    match message {
                        Ok(job) => {
                            results.send(job()).unwrap();
                            done += 1;
                        }
                        Err(_) => break,
                    }
                }
                done
            });
            workers.push(Worker { id, handle: Some(handle) });
        }
        ThreadPool { workers, sender: Some(sender) }
    }

    fn execute<F>(&self, f: F)
    where
        F: FnOnce() -> u64 + Send + 'static,
    {
        self.sender.as_ref().unwrap().send(Box::new(f)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        let mut total = 0;
        for worker in &mut self.workers {
            if let Some(handle) = worker.handle.take() {
                total += handle.join().unwrap();
            }
            println!("shut down worker {}", worker.id);
        }
        println!("pool ran {} jobs", total);
    }
}

fn fib(n: u64) -> u64 {
    if n < 2 { n } else { fib(n - 1) + fib(n - 2) }
}

fn main() {
    let (tx, rx) = mpsc::channel();
    {
        let pool = ThreadPool::new(3, tx);
        for n in [10u64, 12, 5, 15, 1, 8, 13] {
            pool.execute(move || fib(n));
        }
        println!("submitted all jobs");
    }
    let mut results: Vec<u64> = rx.iter().collect();
    results.sort();
    println!("results {:?}", results);
    println!("sum {}", results.iter().sum::<u64>());
}
