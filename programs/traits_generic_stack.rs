use std::collections::HashMap;
use std::fmt::{self, Debug, Display};
use std::hash::Hash;

#[derive(Debug, Clone)]
struct Stack<T> {
    items: Vec<T>,
}

impl<T> Stack<T> {
    fn new() -> Self {
        Stack { items: Vec::new() }
    }
    fn push(&mut self, x: T) -> &mut Self {
        self.items.push(x);
        self
    }
    fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }
    fn peek(&self) -> Option<&T> {
        self.items.last()
    }
    fn len(&self) -> usize {
        self.items.len()
    }
}

impl<T: PartialOrd + Copy> Stack<T> {
    fn max(&self) -> Option<T> {
        let mut it = self.items.iter();
        let mut best = *it.next()?;
        for &x in it {
            if x > best {
                best = x;
            }
        }
        Some(best)
    }
}

impl<T: Display> Display for Stack<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: Vec<String> = self.items.iter().map(|x| x.to_string()).collect();
        write!(f, "[{}>", s.join(" "))
    }
}

struct Pair<A, B> {
    a: A,
    b: B,
}

impl<A: Clone, B: Clone> Pair<A, B> {
    fn swap(&self) -> Pair<B, A> {
        Pair { a: self.b.clone(), b: self.a.clone() }
    }
}

impl<A, B> Debug for Pair<A, B>
where
    A: Debug,
    B: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Pair").field("a", &self.a).field("b", &self.b).finish()
    }
}

fn frequencies<T, I>(items: I) -> Vec<(T, usize)>
where
    T: Hash + Eq + Ord + Clone,
    I: IntoIterator<Item = T>,
{
    let mut m: HashMap<T, usize> = HashMap::new();
    for x in items {
        *m.entry(x).or_insert(0) += 1;
    }
    let mut v: Vec<(T, usize)> = m.into_iter().collect();
    v.sort_by(|x, y| y.1.cmp(&x.1).then(x.0.cmp(&y.0)));
    v
}

fn largest_by<T, K: Ord, F: Fn(&T) -> K>(items: &[T], key: F) -> Option<&T> {
    let mut best: Option<&T> = None;
    for it in items {
        match best {
            Some(b) if key(b) >= key(it) => {}
            _ => best = Some(it),
        }
    }
    best
}

fn eval_rpn(expr: &str) -> Result<i64, String> {
    let mut st: Stack<i64> = Stack::new();
    for tok in expr.split_whitespace() {
        if let Ok(n) = tok.parse::<i64>() {
            st.push(n);
            continue;
        }
        let b = st.pop().ok_or("underflow")?;
        let a = st.pop().ok_or_else(|| format!("underflow at {}", tok))?;
        st.push(match tok {
            "+" => a + b,
            "-" => a - b,
            "*" => a * b,
            "/" => a / b,
            _ => return Err(format!("bad token {}", tok)),
        });
    }
    st.pop().ok_or_else(|| "empty".to_string())
}

fn main() {
    let mut s = Stack::new();
    s.push(3).push(9).push(4);
    println!("{} len={} peek={:?} max={:?}", s, s.len(), s.peek(), s.max());
    let mut t = s.clone();
    t.pop();
    println!("{} {}", s, t);
    let words: Stack<&str> = Stack { items: vec!["x", "y"] };
    println!("{} {:?}", words, words);
    let p = Pair { a: 1, b: "one" };
    println!("{:?} {:?}", p, p.swap());
    println!("{:?}", frequencies("mississippi".chars()));
    println!("{:?}", frequencies(vec![3, 1, 3, 2, 1, 3]));
    let names = ["al", "beatrice", "cy", "dorothea"];
    println!("{:?}", largest_by(&names, |s| s.len()));
    println!("{:?}", largest_by(&[5, -9, 3], |x: &i32| x.abs()));
    for e in ["3 4 + 2 *", "1 2 3", "5 1 2 + 4 * + 3 -", "1 +", "2 3 ^"] {
        println!("{} => {:?}", e, eval_rpn(e));
    }
}
