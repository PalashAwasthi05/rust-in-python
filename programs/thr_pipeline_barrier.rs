use std::sync::mpsc;
use std::sync::{Arc, Barrier, Mutex};
use std::thread;

fn main() {
    // A three-stage pipeline: generate -> square -> filter, connected by channels.
    let (gen_tx, gen_rx) = mpsc::channel::<u32>();
    let (sq_tx, sq_rx) = mpsc::channel::<u32>();
    let (out_tx, out_rx) = mpsc::channel::<String>();

    let generator = thread::spawn(move || {
        for i in 1..=10 {
            gen_tx.send(i).unwrap();
        }
    });
    let squarer = thread::spawn(move || {
        for v in gen_rx {
            sq_tx.send(v * v).unwrap();
        }
    });
    let filter = thread::spawn(move || {
        let mut kept = 0;
        for v in sq_rx {
            if v % 2 == 0 {
                out_tx.send(format!("even square {}", v)).unwrap();
                kept += 1;
            }
        }
        kept
    });
    let lines: Vec<String> = out_rx.iter().collect();
    generator.join().unwrap();
    squarer.join().unwrap();
    let kept = filter.join().unwrap();
    for line in &lines {
        println!("{}", line);
    }
    println!("kept {}", kept);

    // Barrier: every thread finishes phase one before any starts phase two.
    let n = 4;
    let barrier = Arc::new(Barrier::new(n));
    let log = Arc::new(Mutex::new(Vec::new()));
    let handles: Vec<_> = (0..n)
        .map(|i| {
            let barrier = Arc::clone(&barrier);
            let log = Arc::clone(&log);
            thread::spawn(move || {
                log.lock().unwrap().push(format!("phase1-{}", i));
                let leader = barrier.wait().is_leader();
                log.lock().unwrap().push(format!("phase2-{}", i));
                leader
            })
        })
        .collect();
    let leaders = handles.into_iter().map(|h| h.join().unwrap()).filter(|&l| l).count();
    let log = log.lock().unwrap();
    let first_phase2 = log.iter().position(|e| e.starts_with("phase2")).unwrap();
    let last_phase1 = log.iter().rposition(|e| e.starts_with("phase1")).unwrap();
    println!("entries {}", log.len());
    println!("all phase1 before phase2: {}", last_phase1 < first_phase2);
    println!("leaders {}", leaders);
}
