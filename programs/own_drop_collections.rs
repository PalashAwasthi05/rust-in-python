use std::collections::{BTreeMap, VecDeque};
use std::fmt;

struct Tag<T: fmt::Debug>(T);
impl<T: fmt::Debug> Drop for Tag<T> {
    fn drop(&mut self) {
        println!("drop {:?}", self.0);
    }
}

struct Node {
    v: i32,
    next: Option<Box<Node>>,
}
impl Drop for Node {
    fn drop(&mut self) {
        println!("node {}", self.v);
    }
}

struct Wrapper {
    inner: Vec<Box<Tag<&'static str>>>,
    name: Tag<String>,
}

fn build(n: i32) -> Option<Box<Node>> {
    let mut head = None;
    for v in (1..=n).rev() {
        head = Some(Box::new(Node { v, next: head }));
    }
    head
}

fn pick(flag: bool) -> Tag<i32> {
    let a = Tag(1);
    let b = Tag(2);
    if flag { a } else { b }
}

fn check(t: &Tag<i32>) -> bool {
    t.0 > 0
}

fn main() {
    println!("-- list");
    {
        let list = build(3);
        println!("built {}", list.as_ref().unwrap().v);
    }
    println!("-- btree");
    {
        let mut m = BTreeMap::new();
        m.insert(3, Tag("three"));
        m.insert(1, Tag("one"));
        m.insert(2, Tag("two"));
        m.insert(2, Tag("two-b"));
        println!("size {}", m.len());
    }
    println!("-- deque");
    {
        let mut d = VecDeque::new();
        d.push_back(Tag('b'));
        d.push_front(Tag('a'));
        d.push_back(Tag('c'));
        d.pop_front();
        println!("popped front");
    }
    println!("-- wrapper");
    {
        let _w = Wrapper { inner: vec![Box::new(Tag("i0")), Box::new(Tag("i1"))], name: Tag("w".to_string()) };
    }
    println!("-- pick");
    let p = pick(true);
    println!("picked {}", p.0);
    let q = pick(false);
    println!("picked {}", q.0);
    println!("-- temp in condition");
    if check(&Tag(5)) {
        println!("in then");
    }
    while check(&Tag(-1)) {}
    println!("-- temp in let");
    let len = Tag(vec![1, 2]).0.len();
    println!("len {}", len);
    let r = &Tag(77);
    println!("extended {}", r.0);
    println!("-- match on temp");
    match Some(Tag(8)) {
        Some(ref t) if t.0 > 10 => println!("big"),
        Some(t) => println!("small {}", t.0),
        None => {}
    }
    println!("-- vec of tuples");
    {
        let v = vec![(Tag(1), Tag(2)), (Tag(3), Tag(4))];
        let mut it = v.into_iter();
        let (a, _) = it.next().unwrap();
        println!("a {}", a.0);
    }
    println!("-- assign whole");
    {
        let mut t = (Tag("x"), Tag("y"));
        t = (Tag("x2"), Tag("y2"));
        println!("assigned {}", t.0 .0);
    }
    println!("end");
}
