use std::collections::BTreeMap;
use std::fmt::{Debug, Display};
use std::ops::Add;

fn largest<T: PartialOrd + Copy>(items: &[T]) -> Option<T> {
    let mut iter = items.iter();
    let mut best = *iter.next()?;
    for &x in iter {
        if x > best { best = x; }
    }
    Some(best)
}

fn sum_all<T>(items: &[T]) -> T
where
    T: Add<Output = T> + Copy + Default,
{
    items.iter().fold(T::default(), |acc, &x| acc + x)
}

fn describe<T: Debug + Display>(label: &str, value: T) -> String {
    format!("{}: display={} debug={:?}", label, value, value)
}

fn group_by_key<T, K, F>(items: Vec<T>, key: F) -> BTreeMap<K, Vec<T>>
where
    K: Ord,
    F: Fn(&T) -> K,
{
    let mut map = BTreeMap::new();
    for item in items {
        map.entry(key(&item)).or_insert_with(Vec::new).push(item);
    }
    map
}

trait Summary {
    fn summarize(&self) -> String;
}

#[derive(Debug)]
struct Article { title: String, words: usize }
#[derive(Debug)]
struct Tweet { user: String, text: String }

impl Summary for Article {
    fn summarize(&self) -> String { format!("{} ({} words)", self.title, self.words) }
}
impl Summary for Tweet {
    fn summarize(&self) -> String { format!("@{}: {}", self.user, self.text) }
}

fn print_all<S: Summary>(items: &[S]) {
    for s in items { println!("  - {}", s.summarize()); }
}

fn pair_summary(a: &impl Summary, b: &impl Summary) -> String {
    format!("{} | {}", a.summarize(), b.summarize())
}

fn main() {
    println!("largest i32: {:?}", largest(&[3, 9, -2, 7]));
    println!("largest f64: {:?}", largest(&[1.5, 0.2, 8.25]));
    println!("largest char: {:?}", largest(&['q', 'z', 'a']));
    println!("largest empty: {:?}", largest::<u8>(&[]));
    println!("sum i64: {}", sum_all(&[1i64, 2, 3, 4]));
    println!("sum f32: {}", sum_all(&[0.5f32, 0.25, 0.125]));
    println!("{}", describe("word", "hi"));
    println!("{}", describe("num", 42u8));

    let words = vec!["apple", "avocado", "banana", "blueberry", "cherry", "kiwi"];
    let grouped = group_by_key(words, |w| w.chars().next().unwrap());
    println!("{:?}", grouped);
    let by_len = group_by_key(vec![10, 3, 255, 42, 7, 1000], |n| n.to_string().len());
    println!("{:?}", by_len);

    let arts = vec![Article { title: "Rust 2021".into(), words: 1200 }, Article { title: "Traits".into(), words: 800 }];
    println!("articles:");
    print_all(&arts);
    let t = Tweet { user: "ferris".into(), text: "crab rave".into() };
    println!("{}", pair_summary(&arts[0], &t));
}
