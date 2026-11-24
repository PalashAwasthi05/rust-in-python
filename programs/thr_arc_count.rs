use std::sync::Arc;
use std::thread;

#[derive(Debug)]
struct Config {
    name: String,
    limit: usize,
}

impl Drop for Config {
    fn drop(&mut self) {
        println!("dropping config {}", self.name);
    }
}

fn main() {
    let config = Arc::new(Config { name: String::from("main"), limit: 3 });
    println!("count at start: {}", Arc::strong_count(&config));
    let handles: Vec<_> = (0..3)
        .map(|i| {
            let c = Arc::clone(&config);
            thread::spawn(move || format!("worker {} sees {} (limit {})", i, c.name, c.limit))
        })
        .collect();
    let mut lines: Vec<String> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    lines.sort();
    for line in &lines {
        println!("{}", line);
    }
    println!("count after joins: {}", Arc::strong_count(&config));
    {
        let extra = config.clone();
        let another = Arc::clone(&extra);
        println!("count with clones: {}", Arc::strong_count(&config));
        println!("ptr_eq: {}", Arc::ptr_eq(&extra, &another));
    }
    println!("count after scope: {}", Arc::strong_count(&config));
    let weak = Arc::downgrade(&config);
    println!("weak count: {}", Arc::weak_count(&config));
    println!("upgrade some: {}", weak.upgrade().is_some());
    let numbers = Arc::new(vec![1, 2, 3]);
    let n2 = Arc::clone(&numbers);
    let sum = thread::spawn(move || n2.iter().sum::<i32>()).join().unwrap();
    println!("sum {} count {}", sum, Arc::strong_count(&numbers));
    match Arc::try_unwrap(numbers) {
        Ok(v) => println!("unwrapped {:?}", v),
        Err(_) => println!("still shared"),
    }
    let held = Arc::clone(&config);
    let keeper = thread::spawn(move || {
        let name = held.name.clone();
        drop(held);
        name
    });
    println!("keeper saw {}", keeper.join().unwrap());
    println!("count before final drop: {}", Arc::strong_count(&config));
    drop(config);
    println!("upgrade after drop: {:?}", weak.upgrade().map(|c| c.limit));
    let mut unique = Arc::new(5);
    *Arc::make_mut(&mut unique) += 1;
    println!("make_mut {}", unique);
    println!("end");
}
