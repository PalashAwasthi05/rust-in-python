use std::collections::{HashMap, HashSet};

fn matcher<'a>(prefix: &'a str) -> impl Fn(&str) -> bool + 'a {
    move |s| s.starts_with(prefix)
}

fn filter_with<'a>(items: &'a [String], pred: Box<dyn Fn(&str) -> bool + 'a>) -> Vec<&'a String> {
    items.iter().filter(|s| pred(s)).collect()
}

struct Deferred {
    tasks: Vec<Box<dyn FnOnce() -> String>>,
}

impl Deferred {
    fn add<F: FnOnce() -> String + 'static>(&mut self, f: F) {
        self.tasks.push(Box::new(f));
    }
    fn run_all(&mut self) -> Vec<String> {
        self.tasks.drain(..).map(|t| t()).collect()
    }
}

fn main() {
    let words: Vec<String> = ["apple", "apricot", "banana", "avocado", "blueberry"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let p = String::from("ap");
    let is_ap = matcher(&p);
    println!("{:?}", words.iter().filter(|w| is_ap(w)).collect::<Vec<_>>());
    let min = 6;
    println!("{:?}", filter_with(&words, Box::new(move |s| s.len() > min)));

    let priority: HashMap<&str, i32> = [("banana", 1), ("apple", 2)].into_iter().collect();
    let mut sorted = words.clone();
    sorted.sort_by_key(|w| (priority.get(w.as_str()).copied().unwrap_or(99), w.len()));
    println!("{:?}", sorted);
    sorted.sort_unstable_by(|a, b| b.cmp(a));
    println!("{:?}", sorted);

    let banned: HashSet<char> = "bv".chars().collect();
    let mut kept = words.clone();
    kept.retain(|w| !w.chars().any(|c| banned.contains(&c)));
    println!("{:?}", kept);

    let mut d = Deferred { tasks: Vec::new() };
    let owned = String::from("payload");
    d.add(move || format!("got {}", owned));
    let n = 41;
    d.add(move || format!("n+1 = {}", n + 1));
    d.add(|| String::from("static"));
    println!("{:?}", d.run_all());
    println!("left {}", d.tasks.len());

    let once: Box<dyn FnOnce(i32) -> Vec<i32>> = {
        let base = vec![1, 2];
        Box::new(move |x| {
            let mut b = base;
            b.push(x);
            b
        })
    };
    println!("{:?}", once(3));

    fn helper(depth: u32, f: &dyn Fn(u32) -> u32) -> u32 {
        if depth == 0 { f(0) } else { f(depth) + helper(depth - 1, f) }
    }
    let scale = 3;
    println!("{}", helper(4, &|d| d * scale));

    let mut groups: HashMap<bool, Vec<&String>> = HashMap::new();
    for w in &words {
        groups.entry(w.len() % 2 == 0).or_default().push(w);
    }
    println!("{:?} {:?}", groups[&true], groups[&false]);

    let adders: Vec<Box<dyn Fn(i32) -> i32>> = (1..=3).map(|k| Box::new(move |x| x + k) as Box<dyn Fn(i32) -> i32>).collect();
    println!("{:?}", adders.iter().map(|f| f(10)).collect::<Vec<_>>());
    let composed = adders.iter().fold(0, |acc, f| f(acc));
    println!("{}", composed);
}
