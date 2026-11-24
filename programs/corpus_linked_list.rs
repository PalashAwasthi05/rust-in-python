struct Node {
    val: i32,
    next: Option<Box<Node>>,
}

struct List {
    head: Option<Box<Node>>,
    len: usize,
}

impl List {
    fn new() -> Self {
        List { head: None, len: 0 }
    }

    fn push_front(&mut self, val: i32) {
        let old = self.head.take();
        self.head = Some(Box::new(Node { val, next: old }));
        self.len += 1;
    }

    fn pop_front(&mut self) -> Option<i32> {
        self.head.take().map(|node| {
            self.head = node.next;
            self.len -= 1;
            node.val
        })
    }

    fn push_back(&mut self, val: i32) {
        let mut cur = &mut self.head;
        while let Some(node) = cur {
            cur = &mut node.next;
        }
        *cur = Some(Box::new(Node { val, next: None }));
        self.len += 1;
    }

    fn reverse(&mut self) {
        let mut prev: Option<Box<Node>> = None;
        let mut cur = self.head.take();
        while let Some(mut node) = cur {
            cur = node.next.take();
            node.next = prev;
            prev = Some(node);
        }
        self.head = prev;
    }

    fn to_vec(&self) -> Vec<i32> {
        let mut out = Vec::new();
        let mut cur = &self.head;
        while let Some(node) = cur {
            out.push(node.val);
            cur = &node.next;
        }
        out
    }

    fn sum(&self) -> i32 {
        self.to_vec().iter().sum()
    }
}

fn main() {
    let mut list = List::new();
    for i in 1..=5 {
        list.push_front(i * 10);
    }
    list.push_back(99);
    println!("list: {:?} (len {})", list.to_vec(), list.len);
    list.reverse();
    println!("reversed: {:?}", list.to_vec());
    println!("popped: {:?}", list.pop_front());
    println!("popped: {:?}", list.pop_front());
    println!("now: {:?} sum={}", list.to_vec(), list.sum());
    while let Some(v) = list.pop_front() {
        print!("{} ", v);
    }
    println!();
    println!("empty pop: {:?}, len {}", list.pop_front(), list.len);
}
