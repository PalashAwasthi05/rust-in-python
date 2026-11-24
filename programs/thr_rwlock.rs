use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread;

fn main() {
    let config = Arc::new(RwLock::new(HashMap::new()));
    {
        let mut w = config.write().unwrap();
        w.insert("threads", 4);
        w.insert("retries", 3);
    }
    let mut handles = Vec::new();
    for i in 0..3 {
        let config = Arc::clone(&config);
        handles.push(thread::spawn(move || {
            let r = config.read().unwrap();
            let threads = r.get("threads").copied().unwrap_or(0);
            let retries = r["retries"];
            threads * 100 + retries * 10 + i
        }));
    }
    let mut results: Vec<i32> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    results.sort();
    println!("{:?}", results);

    {
        let writer = Arc::clone(&config);
        thread::spawn(move || {
            let mut w = writer.write().unwrap();
            *w.entry("retries").or_insert(0) += 5;
            w.insert("timeout", 30);
        })
        .join()
        .unwrap();
    }
    let r = config.read().unwrap();
    let mut keys: Vec<_> = r.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
    keys.sort();
    println!("{}", keys.join(", "));
    drop(r);

    let lock = RwLock::new(vec![1, 2, 3]);
    {
        let a = lock.read().unwrap();
        let b = lock.read().unwrap();
        println!("two readers: {} {}", a.len(), b[2]);
        println!("write blocked: {}", lock.try_write().is_err());
    }
    lock.write().unwrap().push(4);
    println!("{:?}", *lock.read().unwrap());
    {
        let _w = lock.write().unwrap();
        println!("read blocked: {}", lock.try_read().is_err());
    }
    println!("read free: {}", lock.try_read().is_ok());
    let v = lock.into_inner().unwrap();
    println!("into_inner sum {}", v.iter().sum::<i32>());
}
