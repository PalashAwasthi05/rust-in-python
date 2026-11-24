struct Collatz {
    current: u64,
    done: bool,
}

impl Collatz {
    fn new(start: u64) -> Self { Collatz { current: start, done: false } }
}

impl Iterator for Collatz {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        if self.done { return None; }
        let v = self.current;
        if v == 1 {
            self.done = true;
        } else if v % 2 == 0 {
            self.current = v / 2;
        } else {
            self.current = 3 * v + 1;
        }
        Some(v)
    }
}

struct Fib { a: u64, b: u64 }

impl Iterator for Fib {
    type Item = u64;
    fn next(&mut self) -> Option<Self::Item> {
        let r = self.a;
        self.a = self.b;
        self.b = r.checked_add(self.b)?;
        Some(r)
    }
}

struct Countdown(u32);

impl Iterator for Countdown {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        if self.0 == 0 { None } else { self.0 -= 1; Some(self.0 + 1) }
    }
}

struct Grid { w: usize, h: usize }

impl Grid {
    fn cells(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        (0..self.h).flat_map(move |y| (0..self.w).map(move |x| (x, y)))
    }
}

fn main() {
    let seq: Vec<u64> = Collatz::new(6).collect();
    println!("collatz(6): {:?}", seq);
    println!("collatz(27) steps: {}, peak: {}", Collatz::new(27).count() - 1, Collatz::new(27).max().unwrap());
    let longest = (1..30u64).max_by_key(|&n| (Collatz::new(n).count(), std::cmp::Reverse(n))).unwrap();
    println!("longest chain under 30 starts at {}", longest);
    let fibs: Vec<u64> = Fib { a: 0, b: 1 }.take(12).collect();
    println!("fib: {:?}", fibs);
    let even_sum: u64 = Fib { a: 0, b: 1 }.take_while(|&x| x < 4_000_000).filter(|x| x % 2 == 0).sum();
    println!("even fib sum < 4M: {}", even_sum);
    println!("fib terms before u64 overflow: {}", Fib { a: 0, b: 1 }.count());
    let first_big = Fib { a: 0, b: 1 }.enumerate().find(|(_, v)| v.to_string().len() >= 10);
    println!("first 10-digit fib: {:?}", first_big);

    let launch: Vec<String> = Countdown(5).map(|n| n.to_string()).chain(std::iter::once("liftoff".to_string())).collect();
    println!("{}", launch.join("... "));
    let pairs: Vec<(u32, char)> = Countdown(3).zip("abc".chars()).collect();
    println!("zipped: {:?}", pairs);
    let mut cd = Countdown(4).peekable();
    while let Some(n) = cd.next() {
        if let Some(next) = cd.peek() { print!("{}>{} ", n, next); } else { println!("{} end", n); }
    }

    let g = Grid { w: 3, h: 2 };
    let cells: Vec<_> = g.cells().collect();
    println!("cells: {:?}", cells);
    let diag: Vec<_> = g.cells().filter(|(x, y)| x == y).collect();
    println!("diagonal: {:?}, sum of x*y: {}", diag, g.cells().map(|(x, y)| x * y).sum::<usize>());
    let by_ref_total: u32 = {
        let mut it = Countdown(6);
        let first3: u32 = it.by_ref().take(3).sum();
        let rest: u32 = it.sum();
        first3 * 100 + rest
    };
    println!("by_ref split: {}", by_ref_total);
}
