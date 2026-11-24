// Labeled breaks with values, loop expressions, shadowing, and early returns.
fn find_pair(grid: &[Vec<i32>], target: i32) -> Option<(usize, usize)> {
    let found = 'outer: loop {
        for (r, row) in grid.iter().enumerate() {
            for (c, &v) in row.iter().enumerate() {
                if v == target {
                    break 'outer Some((r, c));
                }
                if v < 0 {
                    continue 'outer;
                }
            }
        }
        break None;
    };
    found
}

fn collatz_len(mut n: u64) -> u32 {
    let mut steps = 0;
    let result = loop {
        if n == 1 {
            break steps;
        }
        n = if n % 2 == 0 { n / 2 } else { 3 * n + 1 };
        steps += 1;
    };
    result
}

fn first_prime_after(n: u32) -> u32 {
    let mut candidate = n + 1;
    'search: loop {
        let mut d = 2;
        while d * d <= candidate {
            if candidate % d == 0 {
                candidate += 1;
                continue 'search;
            }
            d += 1;
        }
        return candidate;
    }
}

fn block_values() -> i32 {
    let x = 5;
    let x = {
        let x = x * 2;
        let y = 'blk: {
            if x > 5 {
                break 'blk x + 100;
            }
            0
        };
        y - 1
    };
    let x = x.to_string();
    let x: i32 = x.len() as i32 + x.parse::<i32>().unwrap();
    x
}

fn parse_all(items: &[&str]) -> Result<Vec<i32>, String> {
    let parse = |s: &str| -> Result<i32, String> {
        let trimmed = s.trim();
        let n: i32 = trimmed.parse().map_err(|_| format!("bad number {:?}", trimmed))?;
        if n < 0 {
            return Err(format!("negative {}", n));
        }
        Ok(n * 2)
    };
    let mut out = vec![];
    for s in items {
        out.push(parse(s)?);
    }
    Ok(out)
}

fn main() {
    let grid = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
    println!("{:?}", find_pair(&grid, 6));
    println!("{:?}", find_pair(&grid, 10));
    for n in [1, 6, 7, 27] {
        println!("collatz({}) = {}", n, collatz_len(n));
    }
    let primes: Vec<u32> = [1, 10, 20, 89].iter().map(|&n| first_prime_after(n)).collect();
    println!("{:?}", primes);
    println!("block = {}", block_values());
    println!("{:?}", parse_all(&["1", " 2 ", "3"]));
    println!("{:?}", parse_all(&["1", "x"]));
    println!("{:?}", parse_all(&["4", "-3"]));
    let mut count = 0;
    let mut i = 0;
    while i < 20 {
        i += 1;
        if i % 3 == 0 { continue; }
        if i > 14 { break; }
        count += i;
    }
    println!("count={} i={}", count, i);
    let label = if count > 50 { "big" } else if count > 20 { "medium" } else { "small" };
    println!("{}", label);
    let v: Vec<i32> = (1..=4).flat_map(|a| (a..=4).map(move |b| a * b)).collect();
    println!("{:?}", v);
}
