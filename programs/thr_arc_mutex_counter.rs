use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];
    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let mut num = counter.lock().unwrap();
                *num += 1;
            }
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
    println!("Result: {}", *counter.lock().unwrap());
    println!("strong count: {}", Arc::strong_count(&counter));

    let shared = Arc::new(Mutex::new(Vec::new()));
    let handles: Vec<_> = (0..5)
        .map(|i| {
            let shared = Arc::clone(&shared);
            thread::spawn(move || {
                shared.lock().unwrap().push(i * i);
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    let mut values = shared.lock().unwrap().clone();
    values.sort();
    println!("squares: {:?}", values);

    let total = Arc::new(Mutex::new(0u64));
    {
        let mut guard = total.lock().unwrap();
        *guard += 40;
        *guard *= 2;
    }
    let t2 = Arc::clone(&total);
    thread::spawn(move || {
        let mut g = t2.lock().unwrap();
        *g += 2;
        drop(g);
        println!("inside thread: {}", t2.lock().unwrap());
    })
    .join()
    .unwrap();
    let inner = Arc::try_unwrap(total).unwrap().into_inner().unwrap();
    println!("final: {}", inner);

    let m = Mutex::new(String::from("abc"));
    {
        let guard = m.lock().unwrap();
        println!("locked while held: {}", m.try_lock().is_err());
        println!("len {}", guard.len());
    }
    println!("unlocked after scope: {}", m.try_lock().is_ok());
    m.lock().unwrap().push_str("def");
    let mut owned = m;
    owned.get_mut().unwrap().push('!');
    println!("{:?}", owned);
    println!("{}", owned.into_inner().unwrap());
}
