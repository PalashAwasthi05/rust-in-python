use List::*;

#[derive(Debug)]
enum List {
    Cons(i32, Box<List>),
    Nil,
}

impl List {
    fn new() -> List { Nil }

    fn prepend(self, elem: i32) -> List { Cons(elem, Box::new(self)) }

    fn len(&self) -> u32 {
        match *self {
            Cons(_, ref tail) => 1 + tail.len(),
            Nil => 0,
        }
    }

    fn sum(&self) -> i32 {
        match self {
            Cons(v, tail) => v + tail.sum(),
            Nil => 0,
        }
    }

    fn stringify(&self) -> String {
        match *self {
            Cons(head, ref tail) => format!("{}, {}", head, tail.stringify()),
            Nil => format!("Nil"),
        }
    }

    fn map(&self, f: &dyn Fn(i32) -> i32) -> List {
        match self {
            Cons(v, tail) => Cons(f(*v), Box::new(tail.map(f))),
            Nil => Nil,
        }
    }

    fn reverse(self) -> List {
        let mut acc = Nil;
        let mut cur = self;
        while let Cons(v, tail) = cur {
            acc = acc.prepend(v);
            cur = *tail;
        }
        acc
    }

    fn nth(&self, n: usize) -> Option<i32> {
        match (self, n) {
            (Nil, _) => None,
            (Cons(v, _), 0) => Some(*v),
            (Cons(_, tail), k) => tail.nth(k - 1),
        }
    }

    fn from_slice(xs: &[i32]) -> List {
        xs.iter().rev().fold(Nil, |acc, &x| acc.prepend(x))
    }
}

fn main() {
    let mut list = List::new();
    list = list.prepend(1);
    list = list.prepend(2);
    list = list.prepend(3);
    println!("linked list has length: {}", list.len());
    println!("{}", list.stringify());
    println!("sum = {}", list.sum());
    let doubled = list.map(&|x| x * 2);
    println!("doubled: {}", doubled.stringify());
    let rev = list.reverse();
    println!("reversed: {}", rev.stringify());
    println!("nth(0)={:?} nth(2)={:?} nth(5)={:?}", rev.nth(0), rev.nth(2), rev.nth(5));
    let from = List::from_slice(&[10, 20, 30, 40]);
    println!("from slice: {} (len {})", from.stringify(), from.len());
    println!("debug: {:?}", List::from_slice(&[7, 8]));
    let empty = List::new();
    println!("empty: {} len {} sum {}", empty.stringify(), empty.len(), empty.sum());
}
