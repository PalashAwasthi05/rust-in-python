use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();
    for id in 0..3 {
        let tx = tx.clone();
        thread::spawn(move || {
            for j in 0..4 {
                tx.send(id * 10 + j).unwrap();
            }
        });
    }
    drop(tx);
    let mut received: Vec<i32> = Vec::new();
    for msg in rx {
        received.push(msg);
    }
    received.sort();
    println!("got {} messages: {:?}", received.len(), received);

    let (tx, rx) = mpsc::channel::<String>();
    let tx2 = tx.clone();
    let a = thread::spawn(move || {
        for word in ["apple", "banana"] {
            tx.send(word.to_string()).unwrap();
        }
    });
    let b = thread::spawn(move || {
        for word in ["cherry", "date"] {
            tx2.send(word.to_uppercase()).unwrap();
        }
    });
    a.join().unwrap();
    b.join().unwrap();
    let mut words: Vec<String> = rx.iter().collect();
    words.sort();
    println!("{:?}", words);

    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let mut sum = 0;
        for value in rx {
            sum += value;
        }
        sum
    });
    for i in 1..=10 {
        tx.send(i).unwrap();
    }
    drop(tx);
    println!("consumer sum = {}", handle.join().unwrap());

    let (tx, rx) = mpsc::channel::<u8>();
    drop(tx);
    match rx.recv() {
        Ok(v) => println!("unexpected {}", v),
        Err(e) => println!("recv error: {:?} / {}", e, e),
    }

    let (req_tx, req_rx) = mpsc::channel::<u32>();
    let (resp_tx, resp_rx) = mpsc::channel::<u32>();
    let server = thread::spawn(move || {
        let mut served = 0;
        while let Ok(n) = req_rx.recv() {
            resp_tx.send(n * n).unwrap();
            served += 1;
        }
        served
    });
    let mut answers = Vec::new();
    for n in [3, 4, 5] {
        req_tx.send(n).unwrap();
        answers.push(resp_rx.recv().unwrap());
    }
    drop(req_tx);
    println!("answers {:?}, served {}", answers, server.join().unwrap());
}
