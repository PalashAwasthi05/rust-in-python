use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError};
use std::thread;

fn producer(tx: SyncSender<u32>, start: u32) {
    for i in start..start + 3 {
        tx.send(i).unwrap();
    }
}

fn drain(rx: &Receiver<u32>) -> Vec<u32> {
    let mut out = Vec::new();
    while let Ok(v) = rx.recv() {
        out.push(v);
    }
    out
}

fn main() {
    let (tx, rx) = mpsc::sync_channel(2);
    let tx_b = tx.clone();
    let h1 = thread::spawn(move || producer(tx, 0));
    let h2 = thread::spawn(move || producer(tx_b, 100));
    let mut values = drain(&rx);
    h1.join().unwrap();
    h2.join().unwrap();
    values.sort();
    println!("{:?}", values);

    let (tx, rx) = mpsc::sync_channel::<&str>(1);
    tx.send("first").unwrap();
    println!("try_send full: {}", tx.try_send("second").is_err());
    println!("try_recv: {:?}", rx.try_recv());
    println!("try_recv empty: {:?}", rx.try_recv());
    drop(tx);
    println!("try_recv after drop: {:?}", rx.try_recv());
    assert_eq!(rx.try_recv(), Err(TryRecvError::Disconnected));

    let (tx, rx) = mpsc::sync_channel(0);
    let h = thread::spawn(move || {
        let mut total = 0;
        for x in rx.iter() {
            total += x;
        }
        total
    });
    for x in [5, 6, 7] {
        tx.send(x).unwrap();
    }
    drop(tx);
    println!("rendezvous total {}", h.join().unwrap());

    let (tx, rx) = mpsc::channel::<i32>();
    drop(rx);
    println!("send after receiver dropped: {}", tx.send(1).is_err());
    match tx.send(7) {
        Err(mpsc::SendError(v)) => println!("returned value {}", v),
        Ok(()) => println!("sent"),
    }

    let (tx, rx) = mpsc::sync_channel::<usize>(3);
    let workers: Vec<_> = (0..3)
        .map(|w| {
            let tx = tx.clone();
            thread::spawn(move || {
                for k in 0..2 {
                    tx.send(w * 100 + k).unwrap();
                }
            })
        })
        .collect();
    drop(tx);
    let mut got: Vec<usize> = rx.into_iter().collect();
    for w in workers {
        w.join().unwrap();
    }
    got.sort_unstable();
    println!("bounded fan-in {:?}", got);
}
