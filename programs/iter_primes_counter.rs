// A prime iterator with internal state, and a counter with a configurable step.
struct Primes {
    found: Vec<u32>,
    candidate: u32,
}

impl Primes {
    fn new() -> Primes {
        Primes { found: Vec::new(), candidate: 1 }
    }
}

impl Iterator for Primes {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        loop {
            self.candidate += 1;
            let c = self.candidate;
            if self.found.iter().take_while(|&&p| p * p <= c).all(|&p| c % p != 0) {
                self.found.push(c);
                return Some(c);
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Counter {
    count: i32,
    step: i32,
    limit: i32,
}

impl Counter {
    fn new(step: i32, limit: i32) -> Self {
        Counter { count: 0, step, limit }
    }
}

impl Iterator for Counter {
    type Item = i32;
    fn next(&mut self) -> Option<i32> {
        if self.count >= self.limit {
            None
        } else {
            self.count += self.step;
            Some(self.count)
        }
    }
}

fn main() {
    let primes: Vec<u32> = Primes::new().take(15).collect();
    println!("{:?}", primes);

    let sum_below_100: u32 = Primes::new().take_while(|&p| p < 100).sum();
    println!("sum of primes below 100: {}", sum_below_100);

    let twin: Vec<(u32, u32)> = Primes::new()
        .zip(Primes::new().skip(1))
        .filter(|(a, b)| b - a == 2)
        .take(6)
        .collect();
    println!("twin primes: {:?}", twin);

    let p200 = Primes::new().nth(199).unwrap();
    println!("200th prime: {}", p200);

    let mut gen = Primes::new();
    let firsts: Vec<u32> = gen.by_ref().take(3).collect();
    let next_three: Vec<u32> = gen.by_ref().take(3).collect();
    println!("{:?} then {:?}; cached {}", firsts, next_three, gen.found.len());

    let c = Counter::new(3, 20);
    let copy = c.clone();
    let doubled: Vec<i32> = c.map(|x| x * 2).collect();
    println!("doubled: {:?}", doubled);
    println!("original state still: {:?}", copy);

    let total: i32 = Counter::new(1, 5)
        .zip(Counter::new(1, 5).skip(1))
        .map(|(a, b)| a * b)
        .filter(|x| x % 3 == 0)
        .sum();
    println!("total: {}", total);

    let product: i64 = Counter::new(1, 10).map(|x| x as i64).product();
    println!("10! = {}", product);

    let mut by_three = Counter::new(3, 10);
    while let Some(v) = by_three.next() {
        print!("{} ", v);
    }
    println!("| after: {:?}", by_three.next());

    let any_big = Counter::new(7, 50).any(|x| x > 40);
    let all_pos = Counter::new(7, 50).all(|x| x > 0);
    let found = Counter::new(4, 50).find(|x| x % 5 == 0);
    println!("{} {} {:?}", any_big, all_pos, found);

    let folded = Counter::new(2, 8).fold(String::new(), |mut acc, x| {
        if !acc.is_empty() {
            acc.push(',');
        }
        acc.push_str(&x.to_string());
        acc
    });
    println!("folded: [{}]", folded);

    let min = Counter::new(5, 30).map(|x| (x - 17).abs()).min();
    println!("closest distance to 17: {:?}", min);
}
