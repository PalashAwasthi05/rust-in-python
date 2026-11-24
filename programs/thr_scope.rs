use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;

fn parallel_sum(data: &[u64], parts: usize) -> u64 {
    let size = (data.len() + parts - 1) / parts;
    thread::scope(|s| {
        let handles: Vec<_> = data.chunks(size).map(|chunk| s.spawn(move || chunk.iter().sum::<u64>())).collect();
        handles.into_iter().map(|h| h.join().unwrap()).sum()
    })
}

fn main() {
    let mut numbers = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let label = String::from("numbers");
    thread::scope(|s| {
        s.spawn(|| {
            println!("{} has {} items", label, numbers.len());
        });
    });
    let total = thread::scope(|s| {
        let a = s.spawn(|| numbers[..4].iter().sum::<i32>());
        let b = s.spawn(|| numbers[4..].iter().sum::<i32>());
        a.join().unwrap() + b.join().unwrap()
    });
    println!("total {}", total);

    thread::scope(|s| {
        let (left, right) = numbers.split_at_mut(4);
        s.spawn(move || {
            for x in left.iter_mut() {
                *x *= 10;
            }
        });
        s.spawn(move || {
            for x in right.iter_mut() {
                *x += 100;
            }
        });
    });
    println!("{:?}", numbers);

    let mut counts = [0usize; 3];
    thread::scope(|s| {
        for (i, slot) in counts.iter_mut().enumerate() {
            s.spawn(move || {
                *slot = (i + 1) * 7;
            });
        }
    });
    println!("{:?}", counts);

    let hits = AtomicU32::new(0);
    thread::scope(|s| {
        for _ in 0..4 {
            s.spawn(|| {
                hits.fetch_add(5, Ordering::SeqCst);
            });
        }
    });
    println!("hits {}", hits.load(Ordering::SeqCst));

    let big: Vec<u64> = (1..=1000).collect();
    println!("parallel sum {}", parallel_sum(&big, 4));
    println!("label still usable: {}", label);
}
