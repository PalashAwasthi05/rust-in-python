fn main() {
    let mut v = vec![5, 3, 8, 1, 9, 2];
    v.sort();
    println!("{:?} {:?} {:?}", v, v.first(), v.binary_search(&8));
    v.sort_by(|a, b| b.cmp(a));
    println!("{:?}", v);
    v.insert(1, 7);
    let r = v.remove(0);
    println!("{} {:?} {}", r, v, v.contains(&7));
    v.retain(|&x| x % 2 == 1);
    println!("{:?}", v);
    let mut w: Vec<i32> = (1..=10).collect();
    w.reverse();
    w.truncate(5);
    w.extend(vec![100, 200]);
    w.extend_from_slice(&[7, 7]);
    w.dedup();
    println!("{:?} {}", w, w.len());
    let drained: Vec<i32> = w.drain(1..3).collect();
    println!("{:?} {:?}", drained, w);
    for win in w.windows(2) { print!("{:?}", win); }
    println!();
    for c in w.chunks(3) { print!("{:?};", c); }
    println!();
    let words = vec!["apple", "fig", "banana"];
    let mut sorted = words.clone();
    sorted.sort_by_key(|s| s.len());
    println!("{:?} {}", sorted, words.join("+"));
    let nested = vec![vec![1, 2], vec![3]];
    println!("{:?}", nested.concat());
    let (a, b) = v.split_at(1);
    println!("{:?} {:?}", a, b);
    let mut arr = [3, 1, 2];
    arr.sort();
    arr.swap(0, 2);
    println!("{:?} {:?} {:?}", arr, arr.iter().rev().collect::<Vec<_>>(), arr.get(5));
    if let Some(x) = v.last_mut() { *x += 1000; }
    println!("{:?}", v);
    let total: i32 = (1..=4).map(|x| x * x).sum();
    let evens: Vec<u32> = (0..10).step_by(3).collect();
    println!("{} {:?} {:?}", total, evens, (0..5).rev().collect::<Vec<_>>());
    let r = 2..6;
    println!("{:?} {} {}", r, r.contains(&3), (1..=3).contains(&4));
}
