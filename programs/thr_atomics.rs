use std::sync::atomic::{AtomicBool, AtomicI32, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

static GLOBAL_COUNT: AtomicUsize = AtomicUsize::new(0);

fn bump() -> usize {
    GLOBAL_COUNT.fetch_add(1, Ordering::SeqCst)
}

fn main() {
    let hits = Arc::new(AtomicUsize::new(0));
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let hits = Arc::clone(&hits);
            thread::spawn(move || {
                for _ in 0..250 {
                    hits.fetch_add(1, Ordering::SeqCst);
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    println!("hits = {}", hits.load(Ordering::SeqCst));

    let ready = Arc::new(AtomicBool::new(false));
    let r2 = Arc::clone(&ready);
    let setter = thread::spawn(move || {
        r2.store(true, Ordering::SeqCst);
    });
    while !ready.load(Ordering::SeqCst) {
        thread::yield_now();
    }
    setter.join().unwrap();
    println!("ready = {}", ready.load(Ordering::SeqCst));

    for _ in 0..5 {
        bump();
    }
    println!("global = {}", GLOBAL_COUNT.load(Ordering::SeqCst));

    let level = AtomicI32::new(10);
    println!("fetch_sub returned {}", level.fetch_sub(3, Ordering::SeqCst));
    println!("swap returned {}", level.swap(-4, Ordering::SeqCst));
    println!("cas ok: {:?}", level.compare_exchange(-4, 99, Ordering::SeqCst, Ordering::SeqCst));
    println!("cas fail: {:?}", level.compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst));
    println!("max: {}", level.fetch_max(150, Ordering::SeqCst));
    println!("now {:?}", level);
    let flag = AtomicBool::new(true);
    println!("fetch_and {} -> {}", flag.fetch_and(false, Ordering::SeqCst), flag.load(Ordering::Relaxed));
    println!("fetch_xor {} -> {}", flag.fetch_xor(true, Ordering::SeqCst), flag.load(Ordering::Acquire));
    println!("into_inner {}", level.into_inner());

    let max_seen = Arc::new(AtomicUsize::new(0));
    let hs: Vec<_> = [3usize, 17, 9, 12]
        .iter()
        .map(|&v| {
            let m = Arc::clone(&max_seen);
            thread::spawn(move || {
                m.fetch_max(v, Ordering::SeqCst);
            })
        })
        .collect();
    for h in hs {
        h.join().unwrap();
    }
    println!("max seen {}", max_seen.load(Ordering::SeqCst));

    let ticket = AtomicUsize::new(0);
    let mut cas_loops = 0;
    loop {
        let current = ticket.load(Ordering::Relaxed);
        match ticket.compare_exchange_weak(current, current + 10, Ordering::AcqRel, Ordering::Relaxed) {
            Ok(_) => break,
            Err(_) => cas_loops += 1,
        }
    }
    println!("ticket {} after {} retries", ticket.load(Ordering::SeqCst), cas_loops);
    let updated = ticket.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |v| if v < 100 { Some(v * 3) } else { None });
    println!("fetch_update {:?} -> {}", updated, ticket.load(Ordering::SeqCst));
}
