use std::thread;

fn square_sum(start: u64, end: u64) -> u64 {
    (start..end).map(|x| x * x).sum()
}

fn main() {
    let mut handles = Vec::new();
    for i in 0..4u64 {
        let handle = thread::spawn(move || {
            let lo = i * 25;
            let hi = lo + 25;
            (i, square_sum(lo, hi))
        });
        handles.push(handle);
    }
    let mut total = 0;
    for handle in handles {
        let (id, part) = handle.join().unwrap();
        println!("worker {} -> {}", id, part);
        total += part;
    }
    println!("total = {}", total);
    assert_eq!(total, square_sum(0, 100));

    let names = vec!["alpha", "beta", "gamma"];
    let lengths: Vec<usize> = names
        .into_iter()
        .map(|name| thread::spawn(move || name.len()))
        .collect::<Vec<_>>()
        .into_iter()
        .map(|h| h.join().unwrap())
        .collect();
    println!("lengths = {:?}", lengths);

    let h = thread::spawn(|| String::from("hello from a thread"));
    let message: String = h.join().expect("thread panicked");
    println!("{}", message);

    let unit = thread::spawn(|| {
        let v: Vec<i32> = (1..=5).collect();
        assert_eq!(v.len(), 5);
    });
    unit.join().unwrap();
    println!("unit thread done");

    let builder = thread::Builder::new().name(String::from("named-worker"));
    let named = builder
        .spawn(|| thread::current().name().map(|s| s.to_string()))
        .unwrap();
    println!("thread name: {:?}", named.join().unwrap());
    println!("main name: {:?}", thread::current().name());

    let panicking = thread::spawn(|| {
        if 1 + 1 == 2 {
            panic!("worker failed");
        }
        5
    });
    let outcome = panicking.join();
    println!("join is_err: {}", outcome.is_err());
}
