#[derive(Debug, Clone)]
struct Fibonacci { curr: u32, next: u32 }

impl Iterator for Fibonacci {
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> {
        let current = self.curr;
        self.curr = self.next;
        self.next = current.checked_add(self.next)?;
        Some(current)
    }
}

fn fibonacci() -> Fibonacci { Fibonacci { curr: 0, next: 1 } }

fn combine_vecs(v: Vec<i32>, u: Vec<i32>) -> impl Iterator<Item = i32> {
    v.into_iter().chain(u.into_iter()).cycle()
}

fn double_positives<'a>(numbers: &'a Vec<i32>) -> impl Iterator<Item = i32> + 'a {
    numbers.iter().filter(|x| x > &&0).map(|x| x * 2)
}

fn parse_csv_document(src: &str) -> Result<Vec<Vec<i32>>, std::num::ParseIntError> {
    src.lines()
        .map(|line| line.split(',').map(|entry| entry.trim().parse::<i32>()).collect())
        .collect()
}

fn describe(items: impl IntoIterator<Item = impl std::fmt::Display>) -> String {
    items.into_iter().map(|i| i.to_string()).collect::<Vec<_>>().join("/")
}

#[derive(Debug, Clone, Copy)]
struct Unit;

#[derive(Clone, Debug)]
struct Pair(Box<i32>, Box<i32>);

fn main() {
    let mut sequence = 0..3;
    println!("four consecutive `next` calls on 0..3");
    println!("> {:?}", sequence.next());
    println!("> {:?}", sequence.next());
    println!("> {:?}", sequence.next());
    println!("> {:?}", sequence.next());
    println!("the first four terms of the Fibonacci sequence are: {:?}", fibonacci().take(4).collect::<Vec<_>>());
    println!("the next four terms after skipping four: {:?}", fibonacci().skip(4).take(4).collect::<Vec<_>>());
    println!("terms that fit in u32: {}", fibonacci().count());
    let array = [1u32, 3, 3, 7];
    for i in array.iter() { print!("> {} ", i); }
    println!();
    let mut fib = fibonacci();
    fib.nth(9);
    let snapshot = fib.clone();
    println!("after 10 terms: {:?}; clone continues independently: {:?}", fib.take(2).collect::<Vec<_>>(), snapshot.take(3).collect::<Vec<_>>());

    let v1 = vec![1, 2, 3];
    let v2 = vec![4, 5];
    let mut v3 = combine_vecs(v1, v2);
    let firsts: Vec<i32> = (0..7).filter_map(|_| v3.next()).collect();
    println!("cycled chain: {:?}", firsts);
    let singles = vec![-3, -2, 2, 3];
    let doubles: Vec<i32> = double_positives(&singles).collect();
    println!("doubled positives {:?}", doubles);
    println!("parsed csv {:?}", parse_csv_document("1, 2 ,3\n4,5, 6"));
    println!("bad csv is_err: {}", parse_csv_document("1,x").is_err());
    println!("describe: {} | {}", describe(vec![1, 2, 3]), describe(["x", "y"]));

    let unit = Unit;
    let copied_unit = unit;
    println!("original: {:?}, copy: {:?}", unit, copied_unit);
    let pair = Pair(Box::new(1), Box::new(2));
    println!("original: {:?}", pair);
    let moved_pair = pair;
    let cloned_pair = moved_pair.clone();
    drop(moved_pair);
    println!("clone survives the original: {:?}", cloned_pair);
    let mut a = vec![String::from("a")];
    let b = a.clone();
    a[0].push('!');
    println!("deep clone: a={:?} b={:?}", a, b);
}
