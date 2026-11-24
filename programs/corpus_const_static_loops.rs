const MAX_RETRIES: u32 = 3;
const GRID: [[u8; 4]; 4] = [
    [1, 0, 0, 1],
    [0, 1, 1, 0],
    [0, 1, 9, 0],
    [1, 0, 0, 1],
];
static GREETING: &str = "inventory check";
static PRIMES: [u32; 6] = [2, 3, 5, 7, 11, 13];
const SECONDS_PER_DAY: u64 = 24 * 60 * 60;

const fn square(x: u32) -> u32 { x * x }
const NINE: u32 = square(3);

fn find_target(target: u8) -> Option<(usize, usize)> {
    let mut found = None;
    'rows: for (r, row) in GRID.iter().enumerate() {
        for (c, &cell) in row.iter().enumerate() {
            if cell == target {
                found = Some((r, c));
                break 'rows;
            }
        }
    }
    found
}

fn main() {
    println!("{} (retries={}, nine={}, day={}s)", GREETING, MAX_RETRIES, NINE, SECONDS_PER_DAY);
    println!("target 9 at {:?}, target 7 at {:?}", find_target(9), find_target(7));

    let x = 5;
    let x = x * 2;
    {
        let x = "shadowed in block";
        println!("inner x = {}", x);
    }
    let x = format!("{}!", x);
    println!("outer x = {}", x);
    let input = "42";
    let input: i32 = input.parse().unwrap();
    println!("parsed input + 1 = {}", input + 1);

    let mut attempt = 0;
    let result = loop {
        attempt += 1;
        if attempt * 7 % 5 == 1 || attempt > MAX_RETRIES {
            break attempt * 10;
        }
    };
    println!("loop broke with {} after {} attempts", result, attempt);

    let mut pairs = Vec::new();
    'outer: for i in 1..10 {
        for j in 1..10 {
            if j > i { continue 'outer; }
            if i * j > 20 { break 'outer; }
            if (i + j) % 4 == 0 { pairs.push((i, j)); }
        }
    }
    println!("pairs: {:?}", pairs);

    let first_composite = 'search: {
        for n in 4.. {
            if PRIMES.iter().any(|&p| p != n && n % p == 0) {
                break 'search n;
            }
        }
        0
    };
    println!("first composite: {}", first_composite);

    let mut n = 0u32;
    while n < 100 { n = n * 2 + 1; }
    println!("while ended at {}", n);
    let total: u32 = PRIMES.iter().sum();
    let ones: usize = GRID.iter().flatten().filter(|&&c| c == 1).count();
    println!("sum primes {} grid ones {} last prime {}", total, ones, PRIMES[PRIMES.len() - 1]);
}
