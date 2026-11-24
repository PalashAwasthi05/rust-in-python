#[derive(Debug)]
struct Node<T> { val: T, next: Option<Box<Node<T>>> }

#[derive(Debug)]
struct Stack<T> { head: Option<Box<Node<T>>>, len: usize }

impl<T: std::fmt::Debug + PartialOrd> Stack<T> {
    fn new() -> Self { Stack { head: None, len: 0 } }
    fn push(&mut self, val: T) {
        let next = self.head.take();
        self.head = Some(Box::new(Node { val, next }));
        self.len += 1;
    }
    fn pop(&mut self) -> Option<T> {
        let node = self.head.take()?;
        self.head = node.next;
        self.len -= 1;
        Some(node.val)
    }
    fn peek(&self) -> Option<&T> { self.head.as_ref().map(|n| &n.val) }
    fn peek_mut(&mut self) -> Option<&mut T> { self.head.as_mut().map(|n| &mut n.val) }
    fn reverse(&mut self) {
        let mut prev = None;
        let mut cur = self.head.take();
        while let Some(mut node) = cur {
            cur = node.next.take();
            node.next = prev;
            prev = Some(node);
        }
        self.head = prev;
    }
    fn iter(&self) -> Iter<'_, T> { Iter { next: self.head.as_deref() } }
    fn remove_first(&mut self, target: &T) -> bool {
        let mut cur = &mut self.head;
        loop {
            match cur {
                None => return false,
                Some(node) if node.val == *target => {
                    *cur = node.next.take();
                    self.len -= 1;
                    return true;
                }
                Some(node) => cur = &mut node.next,
            }
        }
    }
}

struct Iter<'a, T> { next: Option<&'a Node<T>> }

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        self.next.map(|n| {
            self.next = n.next.as_deref();
            &n.val
        })
    }
}

fn main() {
    let mut s = Stack::new();
    for w in ["a", "b", "c", "d", "e"] { s.push(w.to_string()); }
    println!("len {} peek {:?}", s.len, s.peek());
    println!("items: {:?}", s.iter().collect::<Vec<_>>());
    if let Some(top) = s.peek_mut() { top.push_str("!!"); }
    println!("after peek_mut: {:?}", s.peek());
    s.reverse();
    println!("reversed: {:?}", s.iter().map(|x| x.as_str()).collect::<Vec<_>>().join(""));
    println!("remove c: {}, remove z: {}", s.remove_first(&"c".to_string()), s.remove_first(&"z".to_string()));
    println!("pop {:?} pop {:?}", s.pop(), s.pop());
    println!("left {:?} len {}", s.iter().collect::<Vec<_>>(), s.len);
    let mut nums: Stack<i32> = Stack::new();
    for i in 1..=6 { nums.push(i * i); }
    let total: i32 = nums.iter().sum();
    let max = nums.iter().max().copied();
    println!("sum {} max {:?}", total, max);
    while let Some(x) = nums.pop() { if x < 10 { break; } }
    println!("stopped with {:?}, empty? {}", nums.peek(), nums.head.is_none());
}
