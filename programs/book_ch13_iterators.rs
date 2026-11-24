// Chapter 13.2-13.3: iterators, adapters, custom Iterator impls, and the minigrep helpers.
#[derive(PartialEq, Debug)]
struct Shoe {
    size: u32,
    style: String,
}

fn shoes_in_size(shoes: Vec<Shoe>, shoe_size: u32) -> Vec<Shoe> {
    shoes.into_iter().filter(|s| s.size == shoe_size).collect()
}

struct Counter {
    count: u32,
}

impl Counter {
    fn new() -> Counter {
        Counter { count: 0 }
    }
}

impl Iterator for Counter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count < 5 {
            self.count += 1;
            Some(self.count)
        } else {
            None
        }
    }
}

struct Config {
    query: String,
    file_path: String,
    ignore_case: bool,
}

impl Config {
    fn build(mut args: impl Iterator<Item = String>, ignore_case: bool) -> Result<Config, &'static str> {
        args.next();
        let query = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a query string"),
        };
        let file_path = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a file path"),
        };
        Ok(Config { query, file_path, ignore_case })
    }
}

fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    contents.lines().filter(|line| line.contains(query)).collect()
}

fn search_case_insensitive<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let query = query.to_lowercase();
    let mut results = Vec::new();
    for line in contents.lines() {
        if line.to_lowercase().contains(&query) {
            results.push(line);
        }
    }
    results
}

struct Fibonacci {
    curr: u64,
    next: u64,
}

impl Iterator for Fibonacci {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        let current = self.curr;
        self.curr = self.next;
        self.next += current;
        Some(current)
    }
}

fn main() {
    let v1 = vec![1, 2, 3];
    let v1_iter = v1.iter();
    for val in v1_iter {
        println!("Got: {val}");
    }

    let mut v1_iter = v1.iter();
    assert_eq!(v1_iter.next(), Some(&1));
    assert_eq!(v1_iter.next(), Some(&2));
    assert_eq!(v1_iter.next(), Some(&3));
    assert_eq!(v1_iter.next(), None);
    let total: i32 = v1.iter().sum();
    println!("total = {total}");
    let v2: Vec<_> = v1.iter().map(|x| x + 1).collect();
    println!("{:?}", v2);

    let shoes = vec![
        Shoe { size: 10, style: String::from("sneaker") },
        Shoe { size: 13, style: String::from("sandal") },
        Shoe { size: 10, style: String::from("boot") },
    ];
    let in_my_size = shoes_in_size(shoes, 10);
    println!("{:?}", in_my_size);
    assert_eq!(
        in_my_size,
        vec![
            Shoe { size: 10, style: String::from("sneaker") },
            Shoe { size: 10, style: String::from("boot") },
        ]
    );

    let mut counter = Counter::new();
    while let Some(n) = counter.next() {
        print!("{n} ");
    }
    println!();
    let sum: u32 = Counter::new()
        .zip(Counter::new().skip(1))
        .map(|(a, b)| a * b)
        .filter(|x| x % 3 == 0)
        .sum();
    println!("zip/skip/map/filter sum = {sum}");
    let collected: Vec<u32> = Counter::new().take(3).collect();
    println!("{:?} max {:?} last {:?} count {}", collected, Counter::new().max(), Counter::new().last(), Counter::new().count());

    let contents = "Rust:\nsafe, fast, productive.\nPick three.\nDuct tape.\nTrust me.";
    println!("{:?}", search("duct", contents));
    println!("{:?}", search_case_insensitive("rUsT", contents));

    let args = vec!["minigrep".to_string(), "the".to_string(), "poem.txt".to_string()];
    match Config::build(args.into_iter(), false) {
        Ok(cfg) => println!("query={} file={} ignore_case={}", cfg.query, cfg.file_path, cfg.ignore_case),
        Err(e) => println!("Problem parsing arguments: {e}"),
    }
    let short = vec!["minigrep".to_string()];
    match Config::build(short.into_iter(), true) {
        Ok(_) => println!("unexpected"),
        Err(e) => println!("Problem parsing arguments: {e}"),
    }

    let fib = Fibonacci { curr: 0, next: 1 };
    let evens: Vec<u64> = fib.filter(|n| n % 2 == 0).take_while(|&n| n < 1000).collect();
    println!("even fibs {:?}", evens);
    let first_big = Fibonacci { curr: 0, next: 1 }.find(|&n| n > 10_000);
    println!("first fib over 10000: {:?}", first_big);
    let pos = Fibonacci { curr: 0, next: 1 }.position(|n| n == 144);
    println!("144 at {:?}", pos);

    let names = ["ann", "bo", "cyd"];
    let joined = names.iter().enumerate().map(|(i, n)| format!("{i}:{n}")).collect::<Vec<_>>().join(",");
    println!("{joined}");
    let (small, big): (Vec<i32>, Vec<i32>) = (1..=10).partition(|n| *n <= 3);
    println!("{:?} {:?}", small, big);
    let folded = (1..=5).fold(String::new(), |acc, n| if acc.is_empty() { n.to_string() } else { acc + "-" + &n.to_string() });
    println!("{folded}");
    println!("{}", (1..=4).product::<u32>());
    println!("{:?}", (1..20).step_by(6).collect::<Vec<_>>());
    println!("{:?}", vec![vec![1, 2], vec![3], vec![]].into_iter().flatten().collect::<Vec<i32>>());
    println!("{:?}", ["a b", "c d e"].iter().flat_map(|s| s.split(' ')).collect::<Vec<_>>());
    println!("{} {}", (1..10).any(|n| n * n == 49), (1..10).all(|n| n < 9));
    let mut it = [3, 1, 4, 1, 5, 9, 2, 6].iter().peekable();
    let mut runs = Vec::new();
    while let Some(&x) = it.next() {
        if let Some(&&y) = it.peek() {
            runs.push(if y > x { '+' } else { '-' });
        }
    }
    println!("{}", runs.into_iter().collect::<String>());
    let mut scores = vec![("b", 3), ("a", 3), ("c", 7)];
    scores.sort_by(|x, y| y.1.cmp(&x.1).then_with(|| x.0.cmp(y.0)));
    println!("{:?}", scores);
    let min = scores.iter().min_by_key(|(_, s)| *s).map(|(n, _)| *n);
    println!("{:?}", min);
    let chained: Vec<i32> = (1..3).chain(7..9).rev().collect();
    println!("{:?}", chained);
    let s: Vec<(usize, char)> = "héllo".char_indices().skip_while(|(_, c)| *c != 'l').collect();
    println!("{:?}", s);
}
