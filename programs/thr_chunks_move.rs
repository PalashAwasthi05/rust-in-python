use std::thread;

fn main() {
    let data: Vec<i64> = (1..=20).collect();
    let chunk_size = 6;
    let mut handles = Vec::new();
    for (index, chunk) in data.chunks(chunk_size).enumerate() {
        let owned: Vec<i64> = chunk.to_vec();
        handles.push(thread::spawn(move || {
            let sum: i64 = owned.iter().sum();
            let max = *owned.iter().max().unwrap();
            (index, owned.len(), sum, max)
        }));
    }
    let mut results: Vec<(usize, usize, i64, i64)> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    results.sort();
    for (index, len, sum, max) in &results {
        println!("chunk {}: len={} sum={} max={}", index, len, sum, max);
    }
    let grand: i64 = results.iter().map(|r| r.2).sum();
    println!("grand total {}", grand);

    let words = vec![String::from("one"), String::from("two"), String::from("three")];
    let handle = thread::spawn(move || {
        let mut upper: Vec<String> = words.iter().map(|w| w.to_uppercase()).collect();
        upper.reverse();
        upper
    });
    let upper = handle.join().unwrap();
    println!("{:?}", upper);

    let mut matrix = vec![vec![0u32; 4]; 3];
    let rows: Vec<thread::JoinHandle<Vec<u32>>> = (0..3u32)
        .map(|r| thread::spawn(move || (0..4u32).map(|c| r * 10 + c).collect()))
        .collect();
    for (r, h) in rows.into_iter().enumerate() {
        matrix[r] = h.join().unwrap();
    }
    for row in &matrix {
        println!("{:?}", row);
    }

    let mut pieces: Vec<Vec<u8>> = (0..3u8).map(|i| vec![i; 3]).collect();
    let last = pieces.pop().unwrap();
    let first = pieces.remove(0);
    let combined = thread::spawn(move || {
        let mut all = first;
        all.extend(last);
        all
    })
    .join()
    .unwrap();
    println!("combined {:?}, left {:?}", combined, pieces);
}
