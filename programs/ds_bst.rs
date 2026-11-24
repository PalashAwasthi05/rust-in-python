// Generic binary search tree with insert, delete, search, traversals.
use std::cmp::Ordering;
use std::fmt::Debug;

#[derive(Debug)]
struct Node<T: Ord> {
    key: T,
    left: Tree<T>,
    right: Tree<T>,
}

#[derive(Debug)]
struct Tree<T: Ord>(Option<Box<Node<T>>>);

impl<T: Ord + Debug + Clone> Tree<T> {
    fn new() -> Self {
        Tree(None)
    }

    fn insert(&mut self, key: T) -> bool {
        let mut cur = self;
        loop {
            match cur.0 {
                None => {
                    cur.0 = Some(Box::new(Node { key, left: Tree(None), right: Tree(None) }));
                    return true;
                }
                Some(ref mut node) => match key.cmp(&node.key) {
                    Ordering::Less => cur = &mut node.left,
                    Ordering::Greater => cur = &mut node.right,
                    Ordering::Equal => return false,
                },
            }
        }
    }

    fn contains(&self, key: &T) -> bool {
        let mut cur = &self.0;
        while let Some(node) = cur {
            match key.cmp(&node.key) {
                Ordering::Less => cur = &node.left.0,
                Ordering::Greater => cur = &node.right.0,
                Ordering::Equal => return true,
            }
        }
        false
    }

    fn min(&self) -> Option<&T> {
        let mut node = self.0.as_ref()?;
        while let Some(left) = node.left.0.as_ref() {
            node = left;
        }
        Some(&node.key)
    }

    fn max(&self) -> Option<&T> {
        let mut node = self.0.as_ref()?;
        while let Some(right) = &node.right.0 {
            node = right;
        }
        Some(&node.key)
    }

    fn take_min(&mut self) -> Option<T> {
        match self.0 {
            None => None,
            Some(ref mut node) if node.left.0.is_some() => node.left.take_min(),
            Some(_) => {
                let node = self.0.take().unwrap();
                let Node { key, right, .. } = *node;
                *self = right;
                Some(key)
            }
        }
    }

    fn remove(&mut self, key: &T) -> bool {
        match self.0.as_mut() {
            None => false,
            Some(node) => match key.cmp(&node.key) {
                Ordering::Less => node.left.remove(key),
                Ordering::Greater => node.right.remove(key),
                Ordering::Equal => {
                    let mut boxed = self.0.take().unwrap();
                    match (boxed.left.0.is_some(), boxed.right.0.is_some()) {
                        (false, false) => {}
                        (true, false) => *self = std::mem::replace(&mut boxed.left, Tree(None)),
                        (false, true) => *self = std::mem::replace(&mut boxed.right, Tree(None)),
                        (true, true) => {
                            let succ = boxed.right.take_min().unwrap();
                            boxed.key = succ;
                            self.0 = Some(boxed);
                        }
                    }
                    true
                }
            },
        }
    }

    fn inorder(&self, out: &mut Vec<T>) {
        if let Some(node) = &self.0 {
            node.left.inorder(out);
            out.push(node.key.clone());
            node.right.inorder(out);
        }
    }

    fn preorder(&self) -> Vec<T> {
        let mut out = Vec::new();
        let mut stack: Vec<&Node<T>> = Vec::new();
        if let Some(root) = self.0.as_deref() {
            stack.push(root);
        }
        while let Some(n) = stack.pop() {
            out.push(n.key.clone());
            if let Some(r) = n.right.0.as_deref() {
                stack.push(r);
            }
            if let Some(l) = n.left.0.as_deref() {
                stack.push(l);
            }
        }
        out
    }

    fn height(&self) -> usize {
        match &self.0 {
            None => 0,
            Some(n) => 1 + n.left.height().max(n.right.height()),
        }
    }

    fn len(&self) -> usize {
        self.0.as_ref().map_or(0, |n| 1 + n.left.len() + n.right.len())
    }

    fn range_count(&self, lo: &T, hi: &T) -> usize {
        match &self.0 {
            None => 0,
            Some(n) => {
                let here = if &n.key >= lo && &n.key <= hi { 1 } else { 0 };
                let left = if &n.key > lo { n.left.range_count(lo, hi) } else { 0 };
                let right = if &n.key < hi { n.right.range_count(lo, hi) } else { 0 };
                here + left + right
            }
        }
    }
}

fn render<T: Ord + Debug>(t: &Tree<T>) -> String {
    match &t.0 {
        None => ".".to_string(),
        Some(n) => {
            if n.left.0.is_none() && n.right.0.is_none() {
                format!("{:?}", n.key)
            } else {
                format!("({:?} {} {})", n.key, render(&n.left), render(&n.right))
            }
        }
    }
}

fn main() {
    let mut t = Tree::new();
    for k in [50, 30, 70, 20, 40, 60, 80, 35, 45, 65] {
        t.insert(k);
    }
    println!("dup insert 40 -> {}", t.insert(40));
    let mut v = Vec::new();
    t.inorder(&mut v);
    println!("inorder {:?}", v);
    println!("preorder {:?}", t.preorder());
    println!("shape {}", render(&t));
    println!("len {} height {} min {:?} max {:?}", t.len(), t.height(), t.min(), t.max());
    println!("contains 45? {} contains 46? {}", t.contains(&45), t.contains(&46));
    println!("in [33, 66]: {}", t.range_count(&33, &66));

    for k in [20, 30, 50, 99, 70] {
        let removed = t.remove(&k);
        let mut v = Vec::new();
        t.inorder(&mut v);
        println!("remove {:>2} -> {:<5} {:?} shape {}", k, removed, v, render(&t));
    }

    let mut words: Tree<String> = Tree::new();
    for w in "pear apple fig banana cherry apple grape".split(' ') {
        let fresh = words.insert(w.to_string());
        if !fresh {
            println!("already had {}", w);
        }
    }
    let mut ws = Vec::new();
    words.inorder(&mut ws);
    println!("words {:?}", ws);
    println!("first {:?} last {:?}", words.min(), words.max());
    let mut drained = Vec::new();
    while let Some(w) = words.take_min() {
        drained.push(w);
    }
    println!("drained {:?} now len {}", drained, words.len());

    let mut degenerate = Tree::new();
    for i in 0..30u32 {
        degenerate.insert(i);
    }
    println!("degenerate height {}", degenerate.height());
}
