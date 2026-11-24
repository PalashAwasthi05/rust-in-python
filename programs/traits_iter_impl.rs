struct Fib {
    a: u64,
    b: u64,
}

impl Iterator for Fib {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        let r = self.a;
        self.a = self.b;
        self.b += r;
        Some(r)
    }
}

fn fib() -> Fib {
    Fib { a: 0, b: 1 }
}

struct Countdown(u32);

impl Iterator for Countdown {
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            None
        } else {
            self.0 -= 1;
            Some(self.0 + 1)
        }
    }
}

struct Collatz {
    n: u64,
    done: bool,
}

impl Iterator for Collatz {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        if self.done {
            return None;
        }
        let cur = self.n;
        if cur == 1 {
            self.done = true;
        } else if cur % 2 == 0 {
            self.n = cur / 2;
        } else {
            self.n = 3 * cur + 1;
        }
        Some(cur)
    }
}

fn collatz(n: u64) -> impl Iterator<Item = u64> {
    Collatz { n, done: false }
}

fn evens_squared(it: impl Iterator<Item = u64>) -> Vec<u64> {
    it.filter(|x| x % 2 == 0).map(|x| x * x).collect()
}

trait RevHack: Iterator + Sized {
    fn rev_hack(self) -> std::vec::IntoIter<Self::Item> {
        let mut v: Vec<Self::Item> = self.collect();
        v.reverse();
        v.into_iter()
    }
}

impl<I: Iterator> RevHack for I {}

fn main() {
    let first: Vec<u64> = fib().take(12).collect();
    println!("{:?}", first);
    let s: u64 = fib().take_while(|&x| x < 4_000_000).filter(|x| x % 2 == 0).sum();
    println!("even fib sum = {}", s);
    println!("{:?}", evens_squared(fib().skip(1).take(10)));
    let cd: Vec<String> = Countdown(5).map(|n| n.to_string()).collect();
    println!("{} liftoff", cd.join("..."));
    println!("countdown sum {}", Countdown(10).sum::<u32>());
    for (i, v) in Countdown(3).enumerate() {
        println!("step {} -> {}", i, v);
    }
    let zipped: Vec<(u32, u64)> = Countdown(4).zip(fib().skip(5)).collect();
    println!("{:?}", zipped);
    let longest = (1..30u64).max_by_key(|&n| collatz(n).count()).unwrap();
    println!("longest collatz under 30: {} ({} steps)", longest, collatz(longest).count());
    println!("{:?}", collatz(6).collect::<Vec<_>>());
    let mut it = fib().step_by(3);
    println!("{:?} {:?} {:?}", it.next(), it.next(), it.next());
    let peak = collatz(27).max().unwrap();
    println!("peak of 27 = {}", peak);
    let pos = fib().position(|x| x > 1000);
    println!("first fib > 1000 at {:?}", pos);
    let chained: Vec<u32> = Countdown(2).chain(Countdown(3).rev_hack()).collect();
    println!("{:?}", chained);
    let mut pk = Countdown(3).peekable();
    while let Some(x) = pk.next() {
        if let Some(nx) = pk.peek() {
            println!("{} then {}", x, nx);
        } else {
            println!("{} last", x);
        }
    }
}
