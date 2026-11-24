use std::collections::HashMap;
use std::fmt::Debug;

trait Monoid {
    fn empty() -> Self;
    fn combine(self, other: Self) -> Self;
}

impl Monoid for i64 {
    fn empty() -> Self {
        0
    }
    fn combine(self, o: Self) -> Self {
        self + o
    }
}

impl Monoid for String {
    fn empty() -> Self {
        String::new()
    }
    fn combine(mut self, o: Self) -> Self {
        self.push_str(&o);
        self
    }
}

impl<T> Monoid for Vec<T> {
    fn empty() -> Self {
        Vec::new()
    }
    fn combine(mut self, o: Self) -> Self {
        self.extend(o);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Max(i32);

impl Monoid for Max {
    fn empty() -> Self {
        Max(i32::MIN)
    }
    fn combine(self, o: Self) -> Self {
        if o.0 > self.0 { o } else { self }
    }
}

fn concat_all<M: Monoid, I: IntoIterator<Item = M>>(items: I) -> M {
    items.into_iter().fold(M::empty(), M::combine)
}

trait Animal {
    fn new(name: &str) -> Self
    where
        Self: Sized;
    fn name(&self) -> String;
    fn kind() -> &'static str
    where
        Self: Sized;
}

#[derive(Debug)]
struct Cat(String);
#[derive(Debug)]
struct Cow(String);

impl Animal for Cat {
    fn new(name: &str) -> Self {
        Cat(name.to_string())
    }
    fn name(&self) -> String {
        self.0.clone()
    }
    fn kind() -> &'static str {
        "cat"
    }
}

impl Animal for Cow {
    fn new(name: &str) -> Self {
        Cow(name.to_uppercase())
    }
    fn name(&self) -> String {
        self.0.clone()
    }
    fn kind() -> &'static str {
        "cow"
    }
}

fn make_herd<A: Animal + Debug>(names: &[&str]) -> Vec<A> {
    names.iter().map(|n| A::new(n)).collect()
}

fn census<A: Animal>(herd: &[A]) -> String {
    format!("{} {}s: {}", herd.len(), A::kind(), herd.iter().map(|a| a.name()).collect::<Vec<_>>().join(","))
}

fn defaults<T: Default + Debug>() -> String {
    format!("{:?}", T::default())
}

#[derive(Default, Debug)]
struct Config {
    verbose: bool,
    level: u8,
    name: String,
    tags: Vec<String>,
}

fn main() {
    println!("{}", concat_all(vec![1i64, 2, 3, 4]));
    println!("{}", concat_all(vec!["ab".to_string(), "cd".into(), "e".into()]));
    println!("{:?}", concat_all(vec![vec![1], vec![2, 3], vec![]]));
    println!("{:?}", concat_all(vec![Max(3), Max(-2), Max(8), Max(5)]));
    println!("{:?}", concat_all(Vec::<Max>::new()));
    let cats: Vec<Cat> = make_herd(&["tom", "kit"]);
    let cows = make_herd::<Cow>(&["bess", "daisy", "moo"]);
    println!("{}", census(&cats));
    println!("{}", census(&cows));
    println!("{:?}", cats);
    println!("{}", defaults::<i32>());
    println!("{}", defaults::<String>());
    println!("{}", defaults::<Vec<u8>>());
    println!("{}", defaults::<(bool, char, f64)>());
    println!("{}", defaults::<Option<u8>>());
    println!("{}", defaults::<Config>());
    let c = Config { level: 3, ..Default::default() };
    println!("{:?}", c);
    let m: HashMap<String, i32> = Default::default();
    println!("{}", m.len());
    println!("{}", <String as Monoid>::empty().is_empty());
}
