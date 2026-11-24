// Generic Stack<T> and two-stack Queue<T> with Iterator, IntoIterator (owned and borrowed), FromIterator.
use std::fmt;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Stack<T> {
    items: Vec<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Stack { items: Vec::new() }
    }
    pub fn push(&mut self, x: T) -> &mut Self {
        self.items.push(x);
        self
    }
    pub fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }
    pub fn peek(&self) -> Option<&T> {
        self.items.last()
    }
    pub fn len(&self) -> usize {
        self.items.len()
    }
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    pub fn iter(&self) -> StackIter<'_, T> {
        StackIter { stack: self, pos: self.items.len() }
    }
}

pub struct StackIter<'a, T> {
    stack: &'a Stack<T>,
    pos: usize,
}

impl<'a, T> Iterator for StackIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        if self.pos == 0 {
            None
        } else {
            self.pos -= 1;
            Some(&self.stack.items[self.pos])
        }
    }
}

pub struct StackIntoIter<T>(Stack<T>);

impl<T> Iterator for StackIntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.0.pop()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.0.len(), Some(self.0.len()))
    }
}

impl<T> IntoIterator for Stack<T> {
    type Item = T;
    type IntoIter = StackIntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        StackIntoIter(self)
    }
}

impl<'a, T> IntoIterator for &'a Stack<T> {
    type Item = &'a T;
    type IntoIter = StackIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T> FromIterator<T> for Stack<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut s = Stack::new();
        for x in iter {
            s.push(x);
        }
        s
    }
}

impl<T: fmt::Display> fmt::Display for Stack<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let parts: Vec<String> = self.items.iter().map(|x| x.to_string()).collect();
        write!(f, "[{}>", parts.join(" "))
    }
}

#[derive(Debug, Default)]
pub struct Queue<T> {
    inbox: Vec<T>,
    outbox: Vec<T>,
}

impl<T> Queue<T> {
    pub fn new() -> Self {
        Queue { inbox: Vec::new(), outbox: Vec::new() }
    }
    pub fn enqueue(&mut self, x: T) {
        self.inbox.push(x);
    }
    fn shift(&mut self) {
        if self.outbox.is_empty() {
            while let Some(x) = self.inbox.pop() {
                self.outbox.push(x);
            }
        }
    }
    pub fn dequeue(&mut self) -> Option<T> {
        self.shift();
        self.outbox.pop()
    }
    pub fn front(&mut self) -> Option<&T> {
        self.shift();
        self.outbox.last()
    }
    pub fn len(&self) -> usize {
        self.inbox.len() + self.outbox.len()
    }
    pub fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        self.outbox.iter().rev().chain(self.inbox.iter())
    }
}

impl<T> Iterator for Queue<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.dequeue()
    }
}

impl<T> Extend<T> for Queue<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for x in iter {
            self.enqueue(x);
        }
    }
}

fn balanced(s: &str) -> Result<(), (usize, char)> {
    let mut st: Stack<(usize, char)> = Stack::new();
    for (i, c) in s.chars().enumerate() {
        match c {
            '(' | '[' | '{' => {
                st.push((i, c));
            }
            ')' | ']' | '}' => {
                let want = match c {
                    ')' => '(',
                    ']' => '[',
                    _ => '{',
                };
                match st.pop() {
                    Some((_, open)) if open == want => {}
                    _ => return Err((i, c)),
                }
            }
            _ => {}
        }
    }
    match st.pop() {
        None => Ok(()),
        Some(p) => Err(p),
    }
}

fn eval_rpn(expr: &str) -> Result<i64, String> {
    let mut st = Stack::new();
    for tok in expr.split_whitespace() {
        if let Ok(n) = tok.parse::<i64>() {
            st.push(n);
            continue;
        }
        let b = st.pop().ok_or("stack underflow")?;
        let a = st.pop().ok_or_else(|| format!("missing operand for {}", tok))?;
        let r = match tok {
            "+" => a + b,
            "-" => a - b,
            "*" => a * b,
            "/" => {
                if b == 0 {
                    return Err("division by zero".to_string());
                }
                a / b
            }
            other => return Err(format!("unknown token {:?}", other)),
        };
        st.push(r);
    }
    match (st.pop(), st.is_empty()) {
        (Some(v), true) => Ok(v),
        (None, _) => Err("empty expression".into()),
        (Some(_), false) => Err(format!("{} values left over", st.len() + 1)),
    }
}

fn main() {
    let mut s = Stack::new();
    s.push(1).push(2).push(3);
    println!("{} len {} peek {:?}", s, s.len(), s.peek());
    for x in &s {
        print!("{} ", x);
    }
    println!();
    let total: i32 = s.iter().copied().sum();
    println!("sum {}", total);
    let copy = s.clone();
    println!("clone equal: {}", copy == s);
    let owned: Vec<i32> = s.into_iter().collect();
    println!("drained top-first: {:?}", owned);

    let letters: Stack<char> = "abc".chars().collect();
    println!("letters {} {:?}", letters, letters);
    let mut it = letters.into_iter();
    println!("size_hint {:?}", it.size_hint());
    it.next();
    println!("size_hint after next {:?}", it.size_hint());

    for case in ["(a[b]{c})", "(]", "((", "x}", ""] {
        println!("{:12} -> {:?}", format!("{:?}", case), balanced(case));
    }
    for e in ["3 4 + 2 *", "5 1 2 + 4 * + 3 -", "1 +", "4 0 /", "1 2", "2 3 ^", ""] {
        println!("{:20} => {:?}", e, eval_rpn(e));
    }

    let mut q: Queue<String> = Queue::new();
    q.enqueue("a".into());
    q.enqueue("b".into());
    println!("front {:?}", q.front());
    q.enqueue("c".into());
    q.extend(vec!["d".to_string(), "e".to_string()]);
    let snapshot: Vec<&String> = q.iter().collect();
    println!("queue {:?} len {}", snapshot, q.len());
    println!("dequeue {:?}", q.dequeue());
    let upper: Vec<String> = q.by_ref().take(2).map(|s| s.to_uppercase()).collect();
    println!("took {:?}, {} left", upper, q.len());
    let rest: Vec<String> = q.collect();
    println!("rest {:?}", rest);

    let mut numbers: Queue<u32> = Queue::default();
    numbers.extend(1..=10);
    let evens: Vec<u32> = numbers.filter(|n| n % 2 == 0).collect();
    println!("evens {:?}", evens);
}
