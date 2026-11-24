use std::fmt::Debug;

#[derive(Debug)]
struct Stack<T> {
    items: Vec<T>,
}

impl<T: Debug> Stack<T> {
    fn new() -> Self { Stack { items: Vec::new() } }
    fn push(&mut self, item: T) { self.items.push(item); }
    fn pop(&mut self) -> Option<T> { self.items.pop() }
    fn peek(&self) -> Option<&T> { self.items.last() }
    fn len(&self) -> usize { self.items.len() }
    fn is_empty(&self) -> bool { self.items.is_empty() }
}

struct Queue<T> {
    inbox: Vec<T>,
    outbox: Vec<T>,
}

impl<T> Queue<T> {
    fn new() -> Self { Queue { inbox: Vec::new(), outbox: Vec::new() } }
    fn enqueue(&mut self, item: T) { self.inbox.push(item); }
    fn dequeue(&mut self) -> Option<T> {
        if self.outbox.is_empty() {
            while let Some(x) = self.inbox.pop() {
                self.outbox.push(x);
            }
        }
        self.outbox.pop()
    }
    fn len(&self) -> usize { self.inbox.len() + self.outbox.len() }
}

fn balanced(s: &str) -> bool {
    let mut st: Stack<char> = Stack::new();
    for c in s.chars() {
        match c {
            '(' | '[' | '{' => st.push(c),
            ')' | ']' | '}' => {
                let want = match c { ')' => '(', ']' => '[', _ => '{' };
                if st.pop() != Some(want) { return false; }
            }
            _ => {}
        }
    }
    st.is_empty()
}

fn main() {
    let mut s = Stack::new();
    for w in ["alpha", "beta", "gamma"] {
        s.push(w.to_string());
    }
    println!("stack len {} top {:?}", s.len(), s.peek());
    println!("popped {:?}", s.pop());
    println!("stack {:?}", s);

    let mut q = Queue::new();
    for i in 1..=3 { q.enqueue(i); }
    println!("dequeue {:?}", q.dequeue());
    q.enqueue(4);
    q.enqueue(5);
    print!("drain:");
    while let Some(x) = q.dequeue() {
        print!(" {}", x);
    }
    println!(" (remaining {})", q.len());

    for expr in ["(a[b]{c})", "((]", "{[()()]}", "(("] {
        println!("{:<10} balanced={}", expr, balanced(expr));
    }
}
