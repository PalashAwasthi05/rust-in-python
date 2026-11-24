// Treap (randomized BST) with split/merge, an implicit order-statistics size field, and a tiny xorshift RNG.
use std::cmp::Ordering;

struct Rng(u32);

impl Rng {
    fn next(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }
}

#[derive(Debug)]
struct Node {
    key: i32,
    prio: u32,
    size: usize,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

fn size(t: &Option<Box<Node>>) -> usize {
    match t {
        Some(n) => n.size,
        None => 0,
    }
}

fn fix(n: &mut Box<Node>) {
    n.size = 1 + size(&n.left) + size(&n.right);
}

// split into (< key, >= key)
fn split(t: Option<Box<Node>>, key: i32) -> (Option<Box<Node>>, Option<Box<Node>>) {
    match t {
        None => (None, None),
        Some(mut n) => {
            if n.key < key {
                let (l, r) = split(n.right.take(), key);
                n.right = l;
                fix(&mut n);
                (Some(n), r)
            } else {
                let (l, r) = split(n.left.take(), key);
                n.left = r;
                fix(&mut n);
                (l, Some(n))
            }
        }
    }
}

fn merge(a: Option<Box<Node>>, b: Option<Box<Node>>) -> Option<Box<Node>> {
    match (a, b) {
        (None, b) => b,
        (a, None) => a,
        (Some(mut x), Some(mut y)) => {
            if x.prio > y.prio {
                x.right = merge(x.right.take(), Some(y));
                fix(&mut x);
                Some(x)
            } else {
                y.left = merge(Some(x), y.left.take());
                fix(&mut y);
                Some(y)
            }
        }
    }
}

struct Treap {
    root: Option<Box<Node>>,
    rng: Rng,
}

impl Treap {
    fn new(seed: u32) -> Self {
        Treap { root: None, rng: Rng(seed) }
    }

    fn contains(&self, key: i32) -> bool {
        let mut cur = &self.root;
        while let Some(n) = cur {
            match key.cmp(&n.key) {
                Ordering::Equal => return true,
                Ordering::Less => cur = &n.left,
                Ordering::Greater => cur = &n.right,
            }
        }
        false
    }

    fn insert(&mut self, key: i32) -> bool {
        if self.contains(key) {
            return false;
        }
        let node = Box::new(Node { key, prio: self.rng.next(), size: 1, left: None, right: None });
        let (l, r) = split(self.root.take(), key);
        self.root = merge(merge(l, Some(node)), r);
        true
    }

    fn remove(&mut self, key: i32) -> bool {
        let (l, rest) = split(self.root.take(), key);
        let (mid, r) = split(rest, key + 1);
        let found = mid.is_some();
        self.root = merge(l, r);
        found
    }

    fn len(&self) -> usize {
        size(&self.root)
    }

    fn kth(&self, mut k: usize) -> Option<i32> {
        let mut cur = self.root.as_ref();
        while let Some(n) = cur {
            let ls = size(&n.left);
            if k < ls {
                cur = n.left.as_ref();
            } else if k == ls {
                return Some(n.key);
            } else {
                k -= ls + 1;
                cur = n.right.as_ref();
            }
        }
        None
    }

    fn rank(&self, key: i32) -> usize {
        // number of keys strictly less than key
        let mut r = 0;
        let mut cur = &self.root;
        while let Some(n) = cur {
            if key <= n.key {
                cur = &n.left;
            } else {
                r += size(&n.left) + 1;
                cur = &n.right;
            }
        }
        r
    }

    fn to_vec(&self) -> Vec<i32> {
        fn go(t: &Option<Box<Node>>, out: &mut Vec<i32>) {
            if let Some(n) = t {
                go(&n.left, out);
                out.push(n.key);
                go(&n.right, out);
            }
        }
        let mut v = Vec::with_capacity(self.len());
        go(&self.root, &mut v);
        v
    }

    fn check_heap(&self) -> bool {
        fn ok(t: &Option<Box<Node>>) -> bool {
            match t {
                None => true,
                Some(n) => {
                    let l = n.left.as_ref().map_or(true, |c| c.prio <= n.prio && c.key < n.key);
                    let r = n.right.as_ref().map_or(true, |c| c.prio <= n.prio && c.key > n.key);
                    l && r && n.size == 1 + size(&n.left) + size(&n.right) && ok(&n.left) && ok(&n.right)
                }
            }
        }
        ok(&self.root)
    }

    fn depth(&self) -> usize {
        fn d(t: &Option<Box<Node>>) -> usize {
            t.as_ref().map_or(0, |n| 1 + d(&n.left).max(d(&n.right)))
        }
        d(&self.root)
    }
}

fn main() {
    let mut t = Treap::new(2463534242);
    for k in [50, 20, 80, 10, 30, 70, 90, 60, 40, 30] {
        let fresh = t.insert(k);
        if !fresh {
            println!("dup {}", k);
        }
    }
    println!("sorted {:?} len {} heap ok {}", t.to_vec(), t.len(), t.check_heap());
    println!("kth: {:?}", (0..=t.len()).map(|k| t.kth(k)).collect::<Vec<_>>());
    println!("rank(55) = {} rank(10) = {} rank(100) = {}", t.rank(55), t.rank(10), t.rank(100));
    for k in [30, 31, 50, 10] {
        println!("remove {} -> {} now {:?}", k, t.remove(k), t.to_vec());
    }
    println!("heap ok {} median {:?}", t.check_heap(), t.kth(t.len() / 2));

    let mut big = Treap::new(7);
    for i in 0..400 {
        big.insert(i);
    }
    println!("sequential 400: len {} depth {} heap ok {}", big.len(), big.depth(), big.check_heap());
    let removed = (0..400).filter(|i| i % 3 == 0).filter(|&i| big.remove(i)).count();
    println!("removed {} -> len {} kth(100) {:?} rank(200) {}", removed, big.len(), big.kth(100), big.rank(200));

    let (lo, hi) = split(big.root.take(), 200);
    println!("split at 200: {} below, {} at or above", size(&lo), size(&hi));
    big.root = merge(lo, hi);
    println!("merged back {}", big.len());
}
