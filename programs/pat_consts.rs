// const and static items, const fns, static arrays, and lookup tables.
const fn square(x: u32) -> u32 {
    x * x
}

const fn fib(n: u32) -> u64 {
    let mut a = 0u64;
    let mut b = 1u64;
    let mut i = 0;
    while i < n {
        let t = a + b;
        a = b;
        b = t;
        i += 1;
    }
    a
}

const LIMIT: u32 = square(7) + 1;
const FIB_20: u64 = fib(20);
const NAMES: [&str; 4] = ["zero", "one", "two", "three"];
static PRIMES: [u32; 6] = [2, 3, 5, 7, 11, 13];
static GREETING: &str = "hello";
const UNITS: &[(&str, u64)] = &[("KB", 1 << 10), ("MB", 1 << 20), ("GB", 1 << 30)];

struct Config;

impl Config {
    const VERSION: (u8, u8, u8) = (1, 4, 2);
    const MAX_USERS: usize = 3 * 4;

    fn describe() -> String {
        let (major, minor, patch) = Self::VERSION;
        format!("v{}.{}.{} max={}", major, minor, patch, Self::MAX_USERS)
    }
}

fn human(bytes: u64) -> String {
    for &(name, size) in UNITS.iter().rev() {
        if bytes >= size {
            return format!("{:.1} {}", bytes as f64 / size as f64, name);
        }
    }
    format!("{} B", bytes)
}

fn name_of(n: usize) -> &'static str {
    match NAMES.get(n) {
        Some(name) => name,
        None => "many",
    }
}

fn main() {
    println!("LIMIT={} FIB_20={}", LIMIT, FIB_20);
    for i in 0..5 {
        println!("{} -> {}", i, name_of(i));
    }
    let total: u32 = PRIMES.iter().sum();
    println!("primes {:?} total {} len {}", PRIMES, total, PRIMES.len());
    println!("{} world ({} chars)", GREETING, GREETING.len());
    for b in [12, 2048, 5_000_000, 3 << 30] {
        println!("{} = {}", b, human(b));
    }
    println!("{}", Config::describe());
    const LOCAL: i32 = -(1 << 4);
    let arr = [0u8; LIMIT as usize / 10];
    println!("local={} arr_len={}", LOCAL, arr.len());
    match 49 {
        LIMIT => println!("matched limit"),
        x if x == square(7) => println!("matched square {}", x),
        _ => println!("nothing"),
    }
}
