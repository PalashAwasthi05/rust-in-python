#[derive(Debug)]
struct Node {
    val: i32,
    next: Option<Box<Node>>,
}

fn push_back(head: &mut Option<Box<Node>>, val: i32) {
    let mut cur = head;
    while let Some(node) = cur {
        cur = &mut node.next;
    }
    *cur = Some(Box::new(Node { val, next: None }));
}

fn increment_all(head: &mut Option<Box<Node>>) {
    let mut cur = head.as_mut();
    while let Some(node) = cur {
        node.val += 100;
        cur = node.next.as_mut();
    }
}

fn find_mut(head: &mut Option<Box<Node>>, target: i32) -> Option<&mut i32> {
    let mut cur = head.as_deref_mut();
    while let Some(node) = cur {
        if node.val == target {
            return Some(&mut node.val);
        }
        cur = node.next.as_deref_mut();
    }
    None
}

fn to_vec(head: &Option<Box<Node>>) -> Vec<i32> {
    let mut out = vec![];
    let mut cur = head.as_deref();
    while let Some(n) = cur {
        out.push(n.val);
        cur = n.next.as_deref();
    }
    out
}

struct Context<'s, 'c> {
    source: &'s str,
    config: &'c Vec<String>,
}

impl<'s, 'c> Context<'s, 'c> {
    fn words(&self) -> Vec<&'s str> {
        self.source.split_whitespace().filter(|w| !self.config.iter().any(|c| c == w)).collect()
    }
}

fn pick_longer<'a, 'b: 'a>(a: &'a str, b: &'b str) -> &'a str {
    if b.len() > a.len() { b } else { a }
}

fn get_or_first<'a, T>(items: &'a [T], idx: usize) -> Option<&'a T> {
    items.get(idx).or_else(|| items.first())
}

struct Stack<T> {
    items: Vec<T>,
}

impl<T: std::fmt::Debug> Stack<T> {
    fn peek(&self) -> Option<&T> {
        self.items.last()
    }
    fn peek_mut(&mut self) -> Option<&mut T> {
        self.items.last_mut()
    }
    fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        self.items.iter().rev()
    }
}

fn main() {
    let mut list = None;
    for v in [1, 2, 3] {
        push_back(&mut list, v);
    }
    println!("{:?}", to_vec(&list));
    increment_all(&mut list);
    if let Some(v) = find_mut(&mut list, 102) {
        *v = -1;
    }
    println!("{:?}", to_vec(&list));
    println!("{:?}", find_mut(&mut list, 7));

    let stop = vec!["the".to_string(), "a".to_string()];
    let text = String::from("the cat saw a dog");
    let ctx = Context { source: &text, config: &stop };
    let words = ctx.words();
    drop(ctx);
    println!("{:?}", words);

    let outer = String::from("outer-string");
    let res;
    {
        let inner = String::from("in");
        res = pick_longer(&outer, "static-longer-str");
        println!("{}", pick_longer(&inner, &outer));
    }
    println!("{}", res);

    let nums = [10, 20, 30];
    println!("{:?} {:?}", get_or_first(&nums, 1), get_or_first(&nums, 9));
    let empty: [i32; 0] = [];
    println!("{:?}", get_or_first(&empty, 0));

    let mut st = Stack { items: vec![1, 2] };
    if let Some(top) = st.peek_mut() {
        *top *= 50;
    }
    println!("{:?} {:?}", st.peek(), st.iter().collect::<Vec<_>>());
    st.items.push(7);
    let top = st.peek().copied();
    st.items.push(top.unwrap_or(0) + 1);
    println!("{:?}", st.items);
}
