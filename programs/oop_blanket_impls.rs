// `impl<T> Trait for Vec<T>`, impls on foreign types, blanket impls over bounds, extension traits.
use std::collections::HashMap;
use std::fmt::Display;

trait Summary {
    fn summarize(&self) -> String;
    fn headline(&self) -> String {
        format!("[{}]", self.summarize())
    }
}

impl Summary for i32 {
    fn summarize(&self) -> String {
        format!("int {}", self)
    }
}

impl Summary for String {
    fn summarize(&self) -> String {
        format!("{} chars", self.chars().count())
    }
}

impl<T: Summary> Summary for Vec<T> {
    fn summarize(&self) -> String {
        let parts: Vec<String> = self.iter().map(|x| x.summarize()).collect();
        format!("{} items: {}", self.len(), parts.join("; "))
    }
}

impl<T: Summary> Summary for Option<T> {
    fn summarize(&self) -> String {
        match self {
            Some(x) => format!("some {}", x.summarize()),
            None => "nothing".to_string(),
        }
    }
}

impl<A: Summary, B: Summary> Summary for (A, B) {
    fn summarize(&self) -> String {
        format!("pair({}, {})", self.0.summarize(), self.1.summarize())
    }
}

// Extension trait over all iterators of Display items.
trait JoinExt {
    fn join_with(self, sep: &str) -> String;
}

impl<I> JoinExt for I
where
    I: Iterator,
    I::Item: Display,
{
    fn join_with(self, sep: &str) -> String {
        let mut out = String::new();
        for (i, item) in self.enumerate() {
            if i > 0 {
                out.push_str(sep);
            }
            out.push_str(&item.to_string());
        }
        out
    }
}

// Blanket impl for everything Display.
trait Shout {
    fn shout(&self) -> String;
}

impl<T: Display + ?Sized> Shout for T {
    fn shout(&self) -> String {
        format!("{}!", self.to_string().to_uppercase())
    }
}

trait Stats {
    fn mean(&self) -> Option<f64>;
    fn spread(&self) -> i64;
}

impl Stats for [i64] {
    fn mean(&self) -> Option<f64> {
        if self.is_empty() {
            None
        } else {
            Some(self.iter().sum::<i64>() as f64 / self.len() as f64)
        }
    }
    fn spread(&self) -> i64 {
        match (self.iter().max(), self.iter().min()) {
            (Some(a), Some(b)) => a - b,
            _ => 0,
        }
    }
}

trait Tally<K> {
    fn tally(&self) -> Vec<(K, usize)>;
}

impl<K: Clone + Ord + std::hash::Hash> Tally<K> for Vec<K> {
    fn tally(&self) -> Vec<(K, usize)> {
        let mut m: HashMap<K, usize> = HashMap::new();
        for k in self {
            *m.entry(k.clone()).or_default() += 1;
        }
        let mut v: Vec<(K, usize)> = m.into_iter().collect();
        v.sort();
        v
    }
}

fn print_all<T: Summary>(items: &[T]) {
    for item in items {
        println!("{}", item.headline());
    }
}

fn main() {
    println!("{}", 5.summarize());
    println!("{}", vec![1, 2, 3].summarize());
    println!("{}", vec![String::from("héllo"), String::from("ab")].headline());
    println!("{}", vec![vec![1], vec![], vec![7, 8]].summarize());
    println!("{}", Some(vec![4]).summarize());
    println!("{}", (3, None::<i32>).summarize());
    print_all(&[Some(1), None, Some(3)]);

    println!("{}", [1, 2, 3].iter().join_with(", "));
    println!("{}", "a-b-c".split('-').join_with(" + "));
    println!("{}", (1..=4).map(|x| x * x).join_with("|"));

    println!("{}", "hello".shout());
    println!("{}", 42.shout());
    println!("{}", 'x'.shout());
    println!("{}", String::from("abc").shout());

    let data: Vec<i64> = vec![4, 8, 15, 16, 23, 42];
    println!("mean {:?} spread {}", data.mean(), data.spread());
    println!("mean {:?} spread {}", data[..0].mean(), data[..0].spread());
    println!("{:?}", vec!["b", "a", "b", "c", "b"].tally());
    println!("{:?}", vec![3, 1, 3].tally());
}
