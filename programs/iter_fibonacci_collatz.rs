// Stateful user iterators chained with std adapters.
struct Fibonacci {
    curr: u64,
    next: u64,
}

impl Fibonacci {
    fn new() -> Self {
        Fibonacci { curr: 0, next: 1 }
    }
}

impl Iterator for Fibonacci {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        let value = self.curr;
        self.curr = self.next;
        self.next += value;
        Some(value)
    }
}

struct Collatz {
    value: u64,
    done: bool,
}

impl Collatz {
    fn starting_at(value: u64) -> Collatz {
        Collatz { value, done: false }
    }
}

impl Iterator for Collatz {
    type Item = u64;
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let current = self.value;
        if current == 1 {
            self.done = true;
        } else if current % 2 == 0 {
            self.value = current / 2;
        } else {
            self.value = 3 * current + 1;
        }
        Some(current)
    }
}

fn collatz_len(n: u64) -> usize {
    Collatz::starting_at(n).count()
}

fn main() {
    let first: Vec<u64> = Fibonacci::new().take(12).collect();
    println!("first 12: {:?}", first);

    let even_sum: u64 = Fibonacci::new().take_while(|&f| f < 4_000_000).filter(|f| f % 2 == 0).sum();
    println!("even fib sum below 4M: {}", even_sum);

    let big = Fibonacci::new().skip_while(|&f| f < 1000).next().unwrap();
    println!("first fib >= 1000: {}", big);

    let pos = Fibonacci::new().position(|f| f > 10_000).unwrap();
    println!("index of first fib > 10000: {}", pos);

    let path: Vec<String> = Collatz::starting_at(6).map(|v| v.to_string()).collect();
    println!("collatz(6): {}", path.join(" -> "));

    let (best, len) = (1..30u64).map(|n| (n, collatz_len(n))).max_by_key(|&(_, l)| l).unwrap();
    println!("longest chain under 30: {} with {} steps", best, len);

    let peak = Collatz::starting_at(27).max().unwrap();
    println!("peak of collatz(27): {}", peak);

    let pairs: Vec<(u64, u64)> = Fibonacci::new().zip(Fibonacci::new().skip(1)).take(5).collect();
    println!("consecutive pairs: {:?}", pairs);

    let ratios: Vec<String> = Fibonacci::new()
        .skip(1)
        .zip(Fibonacci::new().skip(2))
        .skip(10)
        .take(3)
        .map(|(a, b)| format!("{:.6}", b as f64 / a as f64))
        .collect();
    println!("ratios: {:?}", ratios);

    let mut fib = Fibonacci::new();
    let third = fib.nth(3);
    let after = fib.next();
    println!("nth(3) = {:?}, then {:?}", third, after);

    let odd_steps = Collatz::starting_at(7).filter(|v| v % 2 == 1).count();
    println!("odd values in collatz(7): {}", odd_steps);

    let last = Collatz::starting_at(9).enumerate().last();
    println!("last of collatz(9): {:?}", last);

    for (i, f) in Fibonacci::new().enumerate().step_by(5).take(4) {
        println!("fib[{}] = {}", i, f);
    }
}
