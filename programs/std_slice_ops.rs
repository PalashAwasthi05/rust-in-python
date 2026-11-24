// Slice and Vec operations.
#[derive(Debug, Clone)]
struct Task {
    id: u32,
    priority: u8,
    name: &'static str,
}

fn main() {
    let data = vec![3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5];
    let sums: Vec<i32> = data.windows(3).map(|w| w.iter().sum()).collect();
    println!("window sums: {:?}", sums);
    for chunk in data.chunks(4) {
        println!("chunk {:?}", chunk);
    }
    let exact = data.chunks_exact(3);
    println!("remainder: {:?}", exact.remainder());
    let exact_sums: Vec<i32> = data.chunks_exact(3).map(|c| c.iter().product()).collect();
    println!("exact products: {:?}", exact_sums);
    let rchunks: Vec<&[i32]> = data.rchunks(4).collect();
    println!("rchunks: {:?}", rchunks);
    let (left, right) = data.split_at(4);
    println!("split_at: {:?} | {:?}", left, right);
    println!("first {:?} last {:?}", data.first(), data.last());
    println!("split_first {:?}", data.split_first());
    println!("split_last {:?}", [1].split_last());
    println!("contains 9: {}, starts_with [3,1]: {}, ends_with [5]: {}", data.contains(&9), data.starts_with(&[3, 1]), data.ends_with(&[5]));
    println!("position of 5: {:?}, rposition: {:?}", data.iter().position(|&x| x == 5), data.iter().rposition(|&x| x == 5));
    println!("max_by: {:?}", data.iter().copied().max_by(|a, b| (a % 4).cmp(&(b % 4))));
    println!("min_by_key: {:?}", data.iter().min_by_key(|&&x| (x as i32 - 4).abs()));

    let mut sorted = data.clone();
    sorted.sort_unstable();
    println!("sorted: {:?}", sorted);
    println!("binary_search 5: {:?}", sorted.binary_search(&5).is_ok());
    println!("binary_search 7: {:?}", sorted.binary_search(&7));
    println!("binary_search 0: {:?}", sorted.binary_search(&0));
    sorted.dedup();
    println!("dedup: {:?}", sorted);
    let idx = sorted.partition_point(|&x| x < 4);
    println!("partition_point: {}", idx);

    let mut tasks = vec![
        Task { id: 1, priority: 2, name: "write" },
        Task { id: 2, priority: 1, name: "test" },
        Task { id: 3, priority: 3, name: "ship" },
        Task { id: 4, priority: 1, name: "lint" },
    ];
    tasks.sort_unstable_by_key(|t| t.id);
    tasks.sort_by_key(|t| std::cmp::Reverse(t.priority));
    println!("by priority desc: {:?}", tasks.iter().map(|t| t.name).collect::<Vec<_>>());
    tasks.sort_by(|a, b| a.priority.cmp(&b.priority).then_with(|| b.id.cmp(&a.id)));
    println!("by priority then id desc: {:?}", tasks.iter().map(|t| t.id).collect::<Vec<_>>());
    let found = tasks.binary_search_by_key(&3, |t| t.priority);
    println!("binary_search_by_key: {:?}", found.map(|i| tasks[i].name));
    let found2 = tasks.binary_search_by(|t| t.priority.cmp(&2));
    println!("binary_search_by: {:?}", found2);
    tasks.dedup_by_key(|t| t.priority);
    println!("dedup_by_key: {:?}", tasks.iter().map(|t| t.name).collect::<Vec<_>>());
    tasks.retain(|t| t.name != "write");
    println!("retain: {:?}", tasks);

    let mut v: Vec<i32> = (1..=10).collect();
    let drained: Vec<i32> = v.drain(2..5).collect();
    println!("drained {:?} left {:?}", drained, v);
    let removed: Vec<i32> = v.splice(1..3, [100, 200, 300]).collect();
    println!("splice removed {:?} now {:?}", removed, v);
    v.rotate_left(2);
    println!("rotate_left: {:?}", v);
    v.rotate_right(3);
    println!("rotate_right: {:?}", v);
    v.swap(0, 1);
    v.reverse();
    println!("swap+reverse: {:?}", v);
    if let Some(first) = v.first_mut() {
        *first *= -1;
    }
    if let Some(last) = v.last_mut() {
        *last += 1000;
    }
    println!("first/last mut: {:?}", v);
    v[2..5].fill(0);
    println!("fill: {:?}", v);
    let mut w = vec![1, 2];
    w.extend_from_slice(&v[..2]);
    w.insert(1, 42);
    let r = w.remove(0);
    let sr = w.swap_remove(0);
    println!("w {:?} removed {} swap_removed {}", w, r, sr);
    w.truncate(1);
    w.resize(4, 7);
    println!("resize: {:?}", w);
    let tail = w.split_off(2);
    println!("split_off: {:?} {:?}", w, tail);
    let nested = vec![vec![1, 2], vec![3], vec![]];
    println!("concat: {:?}", nested.concat());
    let words = vec!["a", "b", "c"];
    println!("join: {} {:?}", words.join("-"), [[1, 2], [3, 4]].join(&[0][..]));
    println!("to_vec: {:?}", data[2..4].to_vec());
    for (i, x) in data.iter().rev().enumerate().take(3) {
        println!("rev enumerate {} {}", i, x);
    }
    let mut grid = vec![vec![0; 3]; 2];
    grid[1][2] = 5;
    for row in grid.iter_mut() {
        for cell in row.iter_mut() {
            *cell += 1;
        }
    }
    println!("grid: {:?}", grid);
    let mut arr = [5, 4, 3, 2, 1];
    arr.sort();
    println!("arr: {:?} len {} {}", arr, arr.len(), arr.is_empty());
    let (a, b) = arr.split_at_mut(2);
    a[0] = b[0] * 10;
    println!("split_at_mut: {:?}", arr);
    println!("repeat: {:?}", [1, 2].repeat(3));
    println!("iter sum {} min {:?} max {:?}", arr.iter().sum::<i32>(), arr.iter().min(), arr.iter().max());
    let mut cap = Vec::with_capacity(10);
    cap.push('x');
    println!("cap {:?}", cap);
    println!("contains str: {}", ["x".to_string()].contains(&"x".to_string()));
}
