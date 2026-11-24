use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    let start = Instant::now();
    let handles: Vec<_> = (1..=3u64)
        .map(|i| {
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(5 * i));
                i * 2
            })
        })
        .collect();
    let results: Vec<u64> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    println!("{:?}", results);
    let elapsed = start.elapsed();
    println!("elapsed at least 5ms: {}", elapsed >= Duration::from_millis(5));
    println!("elapsed under a minute: {}", elapsed < Duration::from_secs(60));

    let d = Duration::from_millis(1500);
    println!("{:?} secs={} millis={}", d, d.as_secs(), d.as_millis());
    let e = Duration::from_secs(2) + Duration::from_micros(250);
    println!("{:?} {}", e, e.as_micros());
    println!("{:?}", Duration::from_millis(30) * 2);
    println!("{:?}", Duration::from_nanos(1_500));
    println!("{}", Duration::from_secs_f64(0.25).as_secs_f64());
    println!("{}", Duration::from_millis(2500).subsec_millis());
    let later = Instant::now();
    println!("monotonic: {}", later >= start);
    println!("since: {}", later.duration_since(start) == later - start);
    thread::sleep(Duration::ZERO);

    let (tx, rx) = mpsc::channel::<u8>();
    let timed_out = rx.recv_timeout(Duration::from_millis(10));
    println!("recv_timeout: {:?}", timed_out);
    tx.send(9).unwrap();
    println!("recv_timeout with data: {:?}", rx.recv_timeout(Duration::from_millis(10)));
    let deadline = start + Duration::from_secs(3600);
    println!("deadline in future: {}", deadline > Instant::now());
    println!("done");
}
