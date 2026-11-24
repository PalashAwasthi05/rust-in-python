use std::collections::HashMap;

fn fib(n: u64, memo: &mut HashMap<u64, u128>) -> u128 {
    if n < 2 { return n as u128; }
    if let Some(&v) = memo.get(&n) { return v; }
    let v = fib(n - 1, memo) + fib(n - 2, memo);
    memo.insert(n, v);
    v
}

fn grid_paths(r: usize, c: usize, blocked: &[(usize, usize)], memo: &mut HashMap<(usize, usize), u64>) -> u64 {
    if blocked.contains(&(r, c)) { return 0; }
    if r == 0 && c == 0 { return 1; }
    if let Some(&v) = memo.get(&(r, c)) { return v; }
    let up = if r > 0 { grid_paths(r - 1, c, blocked, memo) } else { 0 };
    let left = if c > 0 { grid_paths(r, c - 1, blocked, memo) } else { 0 };
    let v = up + left;
    memo.insert((r, c), v);
    v
}

fn edit_distance(a: &str, b: &str) -> usize {
    fn go(a: &[char], b: &[char], i: usize, j: usize, memo: &mut HashMap<(usize, usize), usize>) -> usize {
        if i == a.len() { return b.len() - j; }
        if j == b.len() { return a.len() - i; }
        if let Some(&v) = memo.get(&(i, j)) { return v; }
        let v = if a[i] == b[j] {
            go(a, b, i + 1, j + 1, memo)
        } else {
            1 + go(a, b, i + 1, j, memo).min(go(a, b, i, j + 1, memo)).min(go(a, b, i + 1, j + 1, memo))
        };
        memo.insert((i, j), v);
        v
    }
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    go(&a, &b, 0, 0, &mut HashMap::new())
}

fn coin_ways(amount: u32, coins: &[u32], memo: &mut HashMap<(u32, usize), u64>) -> u64 {
    if amount == 0 { return 1; }
    if coins.is_empty() { return 0; }
    let key = (amount, coins.len());
    if let Some(&v) = memo.get(&key) { return v; }
    let with = if coins[0] <= amount { coin_ways(amount - coins[0], coins, memo) } else { 0 };
    let without = coin_ways(amount, &coins[1..], memo);
    memo.insert(key, with + without);
    with + without
}

fn main() {
    let mut memo = HashMap::new();
    for n in [10, 50, 100, 150] {
        println!("fib({}) = {}", n, fib(n, &mut memo));
    }
    println!("memo entries: {}", memo.len());
    let mut gm = HashMap::new();
    println!("grid paths 3x3: {}", grid_paths(3, 3, &[], &mut gm));
    let mut gm = HashMap::new();
    println!("grid paths 16x16: {}", grid_paths(16, 16, &[], &mut gm));
    let mut gm = HashMap::new();
    println!("grid paths 4x4 blocked (2,2): {}", grid_paths(4, 4, &[(2, 2)], &mut gm));
    for (a, b) in [("kitten", "sitting"), ("flaw", "lawn"), ("rust", "rust"), ("", "abc")] {
        println!("edit({:?}, {:?}) = {}", a, b, edit_distance(a, b));
    }
    let mut cm = HashMap::new();
    println!("ways to make 100 from [1,5,10,25,50]: {}", coin_ways(100, &[1, 5, 10, 25, 50], &mut cm));
    let mut cm = HashMap::new();
    println!("ways to make 7 from [2,4]: {}", coin_ways(7, &[2, 4], &mut cm));
}
