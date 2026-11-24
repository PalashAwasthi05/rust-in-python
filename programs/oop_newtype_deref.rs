// Newtype wrappers with Deref / DerefMut, plus Display and From.
use std::fmt;
use std::ops::{Deref, DerefMut};

struct Wrapper(Vec<String>);

impl fmt::Display for Wrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]", self.0.join(", "))
    }
}

impl Deref for Wrapper {
    type Target = Vec<String>;
    fn deref(&self) -> &Vec<String> {
        &self.0
    }
}

impl DerefMut for Wrapper {
    fn deref_mut(&mut self) -> &mut Vec<String> {
        &mut self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Meters(f64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct UserId(u64);

impl From<u64> for UserId {
    fn from(n: u64) -> Self {
        UserId(n)
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "user#{:04}", self.0)
    }
}

struct Email(String);

impl Email {
    fn parse(s: &str) -> Result<Email, String> {
        if s.contains('@') {
            Ok(Email(s.to_lowercase()))
        } else {
            Err(format!("invalid email: {}", s))
        }
    }
}

impl Deref for Email {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

struct Stack<T> {
    inner: Vec<T>,
}

impl<T> Deref for Stack<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.inner
    }
}

impl<T> Stack<T> {
    fn new() -> Self {
        Stack { inner: Vec::new() }
    }
    fn push(&mut self, v: T) {
        self.inner.push(v);
    }
}

fn shout(s: &str) -> String {
    s.to_uppercase()
}

fn total_len(v: &Vec<String>) -> usize {
    v.iter().map(|s| s.len()).sum()
}

fn main() {
    let mut w = Wrapper(vec![String::from("hello"), String::from("world")]);
    println!("w = {}", w);
    w.push("again".to_string());
    println!("len = {}, first = {:?}", w.len(), w.first());
    println!("total = {}", total_len(&w));
    w.sort();
    println!("sorted = {}", w);
    w[0].push('!');
    for s in w.iter_mut() {
        s.insert(0, '>');
    }
    println!("{}", w);
    println!("contains >world: {}", w.contains(&">world".to_string()));

    let a = Meters(3.5);
    let b = Meters(1.25);
    println!("{:?} < {:?}: {}", b, a, b < a);
    println!("sum = {}", Meters(a.0 + b.0).0);

    let mut ids: Vec<UserId> = vec![30u64, 7, 12].into_iter().map(UserId::from).collect();
    ids.sort();
    let shown: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
    println!("{}", shown.join(" "));
    let id: UserId = 99.into();
    println!("{} {:?}", id, id);

    match Email::parse("Alice@Example.COM") {
        Ok(e) => {
            println!("email {} len {} domain {:?}", &*e, e.len(), e.split('@').nth(1));
            println!("shout {}", shout(&e));
            println!("as_ref {}", e.as_ref().starts_with("alice"));
        }
        Err(msg) => println!("{}", msg),
    }
    if let Err(msg) = Email::parse("nope") {
        println!("{}", msg);
    }

    let mut st = Stack::new();
    for i in 1..=5 {
        st.push(i * i);
    }
    println!("stack {:?} last={:?} sum={}", &*st, st.last(), st.iter().sum::<i32>());
    println!("contains 16: {}, slice {:?}", st.contains(&16), &st[1..3]);
}
