// Singly linked list with Option<Box<Node<T>>>, in the style of "Learning Rust With Entirely Too Many Linked Lists".
use std::fmt;

type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}

pub struct List<T> {
    head: Link<T>,
    len: usize,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List { head: None, len: 0 }
    }

    pub fn push(&mut self, elem: T) {
        let new_node = Box::new(Node { elem, next: self.head.take() });
        self.head = Some(new_node);
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        self.head.take().map(|node| {
            self.head = node.next;
            self.len -= 1;
            node.elem
        })
    }

    pub fn peek(&self) -> Option<&T> {
        self.head.as_ref().map(|node| &node.elem)
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|node| &mut node.elem)
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    pub fn reverse(&mut self) {
        let mut prev: Link<T> = None;
        let mut cur = self.head.take();
        while let Some(mut node) = cur {
            cur = node.next.take();
            node.next = prev;
            prev = Some(node);
        }
        self.head = prev;
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter { next: self.head.as_deref() }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut { next: self.head.as_deref_mut() }
    }

    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }

    pub fn push_back(&mut self, elem: T) {
        let mut cursor = &mut self.head;
        while let Some(node) = cursor {
            cursor = &mut node.next;
        }
        *cursor = Some(Box::new(Node { elem, next: None }));
        self.len += 1;
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut cur_link = self.head.take();
        while let Some(mut boxed_node) = cur_link {
            cur_link = boxed_node.next.take();
        }
    }
}

pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.as_deref();
            &node.elem
        })
    }
}

pub struct IterMut<'a, T> {
    next: Option<&'a mut Node<T>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node| {
            self.next = node.next.as_deref_mut();
            &mut node.elem
        })
    }
}

pub struct IntoIter<T>(List<T>);

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.0.pop()
    }
}

impl<T: fmt::Display> fmt::Display for List<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        for (i, x) in self.iter().enumerate() {
            if i > 0 {
                write!(f, " -> ")?;
            }
            write!(f, "{}", x)?;
        }
        write!(f, "]")
    }
}

fn main() {
    let mut list: List<i32> = List::new();
    println!("empty: {} len={} peek={:?}", list.is_empty(), list.len(), list.peek());
    println!("pop on empty: {:?}", list.pop());

    for i in 1..=5 {
        list.push(i * 10);
    }
    println!("after pushes: {} (len {})", list, list.len());
    println!("peek: {:?}", list.peek());

    if let Some(v) = list.peek_mut() {
        *v += 1;
    }
    println!("after peek_mut: {}", list);
    list.peek_mut().map(|v| *v *= 2);
    println!("peek after map: {:?}", list.peek());

    println!("pop: {:?}", list.pop());
    println!("pop: {:?}", list.pop());
    println!("now: {} (len {})", list, list.len());

    list.reverse();
    println!("reversed: {}", list);

    for v in list.iter_mut() {
        *v = *v * 100 + 7;
    }
    println!("iter_mut: {}", list);

    let sum: i32 = list.iter().sum();
    let max = list.iter().max();
    println!("sum={} max={:?}", sum, max);
    let evens: Vec<&i32> = list.iter().filter(|x| **x % 2 == 0).collect();
    println!("evens: {:?}", evens);

    list.push_back(1);
    list.push_back(2);
    println!("push_back: {} (len {})", list, list.len());

    let doubled: Vec<i32> = list.into_iter().map(|x| x * 2).collect();
    println!("into_iter doubled: {:?}", doubled);

    let mut words: List<String> = List::new();
    for w in "the quick brown fox".split_whitespace() {
        words.push(w.to_string());
    }
    words.reverse();
    println!("words: {}", words);
    if let Some(first) = words.peek_mut() {
        first.push_str("!!");
    }
    let joined: Vec<String> = words.iter().map(|s| s.to_uppercase()).collect();
    println!("joined: {}", joined.join(" "));
    while let Some(w) = words.pop() {
        print!("{}({}) ", w, w.len());
    }
    println!();
    println!("drained empty: {}", words.is_empty());

    // a long list to exercise the iterative Drop
    let mut big = List::new();
    for i in 0..2000 {
        big.push(i);
    }
    println!("big len {} head {:?}", big.len(), big.peek());
}
