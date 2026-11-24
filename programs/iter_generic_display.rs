// Generic functions over I: IntoIterator with Display, Debug and arithmetic bounds.
use std::collections::BTreeMap;
use std::fmt::{Debug, Display};

fn join_all<I, T>(items: I, sep: &str) -> String
where
    I: IntoIterator<Item = T>,
    T: Display,
{
    let mut out = String::new();
    for (i, item) in items.into_iter().enumerate() {
        if i > 0 {
            out.push_str(sep);
        }
        out.push_str(&item.to_string());
    }
    out
}

fn print_numbered<I>(items: I)
where
    I: IntoIterator,
    I::Item: Debug,
{
    for (i, item) in items.into_iter().enumerate() {
        println!("  {}. {:?}", i + 1, item);
    }
}

fn largest<I: IntoIterator<Item = T>, T: PartialOrd + Copy>(items: I) -> Option<T> {
    let mut it = items.into_iter();
    let mut best = it.next()?;
    for x in it {
        if x > best {
            best = x;
        }
    }
    Some(best)
}

fn total<I>(items: I) -> i64
where
    I: IntoIterator,
    I::Item: Into<i64>,
{
    items.into_iter().map(Into::into).sum()
}

fn table<K: Display, V: Display>(rows: impl IntoIterator<Item = (K, V)>) {
    for (k, v) in rows {
        println!("| {:<6} | {:>5} |", k, v);
    }
}

fn describe(items: impl Iterator<Item = impl Display>) -> Vec<String> {
    items.map(|x| format!("[{}]", x)).collect()
}

struct Celsius(f64);
impl Display for Celsius {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:.1}C", self.0)
    }
}

fn main() {
    println!("{}", join_all(vec![1, 2, 3], ", "));
    println!("{}", join_all(["a", "b"], "-"));
    println!("{}", join_all(&[1.5, 2.25], " / "));
    println!("{}", join_all((1..=4).map(|x| x * x), "+"));
    println!("{}", join_all(vec![Celsius(21.456), Celsius(-3.0)], "; "));
    println!("{}", join_all("xyz".chars().rev(), ""));

    println!("numbered:");
    print_numbered(vec!["alpha", "beta"]);
    print_numbered(Some(42));
    print_numbered(vec![(1, 'a'), (2, 'b')].iter());

    println!("{:?}", largest(vec![3, 9, 2]));
    println!("{:?}", largest([1.5, -2.0]));
    println!("{:?}", largest(Vec::<u8>::new()));
    println!("{:?}", largest("hello".chars()));

    println!("{}", total(vec![1i32, 2, 3]));
    println!("{}", total([200u8, 100]));

    let mut scores = BTreeMap::new();
    scores.insert("ann", 91);
    scores.insert("bob", 78);
    scores.insert("cyd", 85);
    table(scores.iter());
    table(vec![("x", 1.5), ("y", 2.0)]);
    println!("{:?}", describe(1..4));
    println!("{:?}", describe(vec!["p", "q"].into_iter()));
}
