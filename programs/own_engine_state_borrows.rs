use std::collections::HashMap;

#[derive(Debug)]
enum Phase {
    Waiting(u32),
    Active { left: u32, log: Vec<String> },
    Finished,
}

struct Engine {
    phase: Phase,
    items: Vec<u64>,
    cache: HashMap<u64, String>,
    counter: u64,
    names: Vec<String>,
}

impl Engine {
    fn compute(&self) -> u64 {
        self.items.iter().sum::<u64>() + self.counter
    }

    fn step(&mut self) {
        self.items.push(self.compute());
        self.counter += 1;
        self.cache.insert(self.counter, format!("c{}", self.counter));
        let n = self.items.len();
        self.items[n - 1] = self.items[n - 1] % 97;
    }

    fn describe(&mut self, key: u64) -> String {
        if let Some(s) = self.cache.get(&key) {
            return s.clone();
        }
        let made = format!("fresh{}", key);
        self.cache.insert(key, made.clone());
        made
    }

    fn advance(&mut self) {
        match &mut self.phase {
            Phase::Waiting(n) if *n > 0 => *n -= 1,
            Phase::Waiting(_) => self.phase = Phase::Active { left: 2, log: vec![] },
            Phase::Active { left, log } => {
                log.push(format!("left {}", left));
                if *left == 0 {
                    self.phase = Phase::Finished;
                } else {
                    *left -= 1;
                }
            }
            Phase::Finished => {}
        }
    }

    fn longest_name(&self) -> Option<&String> {
        self.names.iter().max_by_key(|n| n.len())
    }

    fn rename_longest(&mut self) {
        if let Some(i) = self.names.iter().position(|n| Some(n) == self.longest_name()) {
            let upper = self.names[i].to_uppercase();
            self.names[i] = upper;
        }
    }
}

fn main() {
    let mut e = Engine {
        phase: Phase::Waiting(1),
        items: vec![1, 2],
        cache: HashMap::new(),
        counter: 0,
        names: vec!["al".into(), "bertha".into(), "cy".into()],
    };
    for _ in 0..4 {
        e.step();
    }
    println!("{:?} counter {}", e.items, e.counter);
    println!("{} {}", e.describe(2), e.describe(10));
    println!("{}", e.describe(10));
    for _ in 0..6 {
        e.advance();
        println!("{:?}", e.phase);
    }
    e.rename_longest();
    println!("{:?}", e.names);

    let mut fib = vec![0u64, 1];
    for i in 2..15 {
        fib.push(fib[i - 1] + fib[i - 2]);
    }
    let last = fib.len() - 1;
    fib[last] = fib[last - 1] * 2;
    println!("{:?}", &fib[10..]);
    let copy = fib.clone();
    fib.extend(copy.iter().take(3));
    fib.extend_from_slice(&copy[..2]);
    println!("{}", fib.len());

    let mut s = String::from("ab");
    s.push_str(&s.clone());
    let t = s.clone() + &s;
    println!("{} {}", s, t);
    let v = &mut fib;
    v.push(v.len() as u64);
    v.truncate(3);
    println!("{:?}", v);

    let mut grid = vec![vec![0; 3]; 3];
    for r in 0..3 {
        for c in 0..3 {
            grid[r][c] = if r == 0 || c == 0 { 1 } else { grid[r - 1][c] + grid[r][c - 1] };
        }
    }
    println!("{:?}", grid);
    let (first, rest) = grid.split_first_mut().unwrap();
    first[0] = rest.iter().map(|row| row[2]).sum();
    println!("{:?}", grid[0]);
}
