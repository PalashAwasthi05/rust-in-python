// Chapter 16: threads, message passing, and shared state (deterministic output only).
use std::sync::mpsc;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;

fn parallel_sum(data: Vec<u64>, chunks: usize) -> u64 {
    let data = Arc::new(data);
    let size = (data.len() + chunks - 1) / chunks;
    let mut handles = Vec::new();
    for i in 0..chunks {
        let data = Arc::clone(&data);
        handles.push(thread::spawn(move || {
            let start = (i * size).min(data.len());
            let end = ((i + 1) * size).min(data.len());
            data[start..end].iter().sum::<u64>()
        }));
    }
    handles.into_iter().map(|h| h.join().unwrap()).sum()
}

fn main() {
    let handle = thread::spawn(|| {
        let mut total = 0;
        for i in 1..10 {
            total += i;
            thread::sleep(Duration::from_millis(1));
        }
        total
    });
    let result = handle.join().unwrap();
    println!("spawned thread computed {result}");

    let v = vec![1, 2, 3];
    let handle = thread::spawn(move || format!("Here's a vector: {v:?}"));
    println!("{}", handle.join().unwrap());

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let val = String::from("hi");
        tx.send(val).unwrap();
    });
    let received = rx.recv().unwrap();
    println!("Got: {received}");

    let (tx, rx) = mpsc::channel();
    let producer = thread::spawn(move || {
        let vals = vec![
            String::from("hi"),
            String::from("from"),
            String::from("the"),
            String::from("thread"),
        ];
        for val in vals {
            tx.send(val).unwrap();
            thread::sleep(Duration::from_millis(1));
        }
    });
    producer.join().unwrap();
    for received in rx {
        println!("Got: {received}");
    }

    let (tx, rx) = mpsc::channel();
    let mut producers = Vec::new();
    for id in 0..3 {
        let tx = tx.clone();
        producers.push(thread::spawn(move || {
            for j in 0..3 {
                tx.send((id, id * 10 + j)).unwrap();
            }
        }));
    }
    drop(tx);
    for p in producers {
        p.join().unwrap();
    }
    let mut all: Vec<(i32, i32)> = rx.iter().collect();
    all.sort();
    println!("{} messages: {:?}", all.len(), all);

    let m = Mutex::new(5);
    {
        let mut num = m.lock().unwrap();
        *num = 6;
    }
    println!("m = {:?}", *m.lock().unwrap());

    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];
    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut num = counter.lock().unwrap();
            *num += 1;
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
    println!("Result: {}", *counter.lock().unwrap());
    println!("strong count back to {}", Arc::strong_count(&counter));

    let log = Arc::new(Mutex::new(Vec::new()));
    let workers: Vec<_> = (0..4)
        .map(|i| {
            let log = Arc::clone(&log);
            thread::spawn(move || {
                log.lock().unwrap().push(i * i);
            })
        })
        .collect();
    for w in workers {
        w.join().unwrap();
    }
    let mut squares = log.lock().unwrap().clone();
    squares.sort();
    println!("squares {:?}", squares);

    let config = Arc::new(RwLock::new(String::from("v1")));
    {
        let r1 = config.read().unwrap();
        let r2 = config.read().unwrap();
        println!("readers see {} and {}", *r1, *r2);
    }
    config.write().unwrap().push_str("-patched");
    println!("after write: {}", config.read().unwrap());

    let data: Vec<u64> = (1..=100).collect();
    println!("parallel sum = {}", parallel_sum(data, 4));

    let mut numbers = vec![1, 2, 3, 4];
    thread::scope(|s| {
        s.spawn(|| {
            println!("scoped thread sees len {}", numbers.len());
        });
    });
    numbers.push(5);
    println!("after scope {:?}", numbers);
}
