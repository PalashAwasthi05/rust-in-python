// Chapter 15.1-15.3: Box<T>, recursive cons lists, Deref, deref coercion, and Drop.
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
enum List {
    Cons(i32, Box<List>),
    Nil,
}

use crate::List::{Cons, Nil};

impl List {
    fn sum(&self) -> i32 {
        match self {
            Cons(v, rest) => v + rest.sum(),
            Nil => 0,
        }
    }

    fn to_vec(&self) -> Vec<i32> {
        let mut out = Vec::new();
        let mut cur = self;
        while let Cons(v, next) = cur {
            out.push(*v);
            cur = next;
        }
        out
    }
}

fn from_slice(values: &[i32]) -> List {
    match values {
        [] => Nil,
        [first, rest @ ..] => Cons(*first, Box::new(from_slice(rest))),
    }
}

struct MyBox<T>(T);

impl<T> MyBox<T> {
    fn new(x: T) -> MyBox<T> {
        MyBox(x)
    }
}

impl<T> Deref for MyBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for MyBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

fn hello(name: &str) {
    println!("Hello, {name}!");
}

struct CustomSmartPointer {
    data: String,
}

impl Drop for CustomSmartPointer {
    fn drop(&mut self) {
        println!("Dropping CustomSmartPointer with data `{}`!", self.data);
    }
}

struct Tree {
    label: &'static str,
    children: Vec<Box<Tree>>,
}

impl Tree {
    fn depth(&self) -> usize {
        1 + self.children.iter().map(|c| c.depth()).max().unwrap_or(0)
    }
    fn labels(&self, out: &mut Vec<&'static str>) {
        out.push(self.label);
        for c in &self.children {
            c.labels(out);
        }
    }
}

trait Shape {
    fn area(&self) -> f64;
}
struct Sq(f64);
struct Circle(f64);
impl Shape for Sq {
    fn area(&self) -> f64 {
        self.0 * self.0
    }
}
impl Shape for Circle {
    fn area(&self) -> f64 {
        3.0 * self.0 * self.0
    }
}

fn main() {
    let b = Box::new(5);
    println!("b = {b}");
    let doubled = *b * 2;
    println!("doubled = {doubled}");

    let list = Cons(1, Box::new(Cons(2, Box::new(Cons(3, Box::new(Nil))))));
    println!("{:?}", list);
    println!("sum {} vec {:?}", list.sum(), list.to_vec());
    let built = from_slice(&[4, 5, 6, 7]);
    println!("{:?} sum {}", built.to_vec(), built.sum());

    let x = 5;
    let y = &x;
    assert_eq!(5, x);
    assert_eq!(5, *y);
    let y = Box::new(x);
    assert_eq!(5, *y);
    let y = MyBox::new(x);
    assert_eq!(5, *y);
    println!("deref checks passed: {}", *y + 1);

    let m = MyBox::new(String::from("Rust"));
    hello(&m);
    hello(&(*m)[..]);
    println!("len through deref = {}", m.len());
    let mut mb = MyBox::new(vec![1, 2]);
    mb.push(3);
    (*mb).push(4);
    println!("{:?} first {:?}", *mb, mb.first());

    let c = CustomSmartPointer {
        data: String::from("my stuff"),
    };
    let d = CustomSmartPointer {
        data: String::from("other stuff"),
    };
    println!("CustomSmartPointers created.");
    let e = CustomSmartPointer {
        data: String::from("some data"),
    };
    println!("CustomSmartPointer created.");
    drop(e);
    println!("CustomSmartPointer dropped before the end of main.");
    {
        let _inner = CustomSmartPointer {
            data: String::from("inner scope"),
        };
        println!("leaving inner scope");
    }
    let boxed = Box::new(CustomSmartPointer {
        data: String::from("boxed"),
    });
    let moved = boxed;
    println!("moved box holds {}", moved.data);
    let _ = CustomSmartPointer {
        data: String::from("wildcard temporary"),
    };
    let v = vec![
        CustomSmartPointer { data: String::from("vec 0") },
        CustomSmartPointer { data: String::from("vec 1") },
    ];
    println!("vec has {} pointers; c={} d={}", v.len(), c.data, d.data);

    let tree = Tree {
        label: "root",
        children: vec![
            Box::new(Tree { label: "a", children: vec![Box::new(Tree { label: "a1", children: vec![] })] }),
            Box::new(Tree { label: "b", children: vec![] }),
        ],
    };
    let mut labels = Vec::new();
    tree.labels(&mut labels);
    println!("depth {} labels {:?}", tree.depth(), labels);

    let shapes: Vec<Box<dyn Shape>> = vec![Box::new(Sq(2.0)), Box::new(Circle(1.0))];
    let total: f64 = shapes.iter().map(|s| s.area()).sum();
    println!("total area {total}");
    println!("end of main");
}
