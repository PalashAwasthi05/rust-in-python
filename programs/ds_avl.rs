// AVL tree map with rotations, generic over K: Ord and V.
use std::cmp::{max, Ordering};
use std::fmt::Display;

type Link<K, V> = Option<Box<AvlNode<K, V>>>;

struct AvlNode<K, V> {
    key: K,
    value: V,
    height: i32,
    left: Link<K, V>,
    right: Link<K, V>,
}

fn height<K, V>(link: &Link<K, V>) -> i32 {
    link.as_ref().map_or(0, |n| n.height)
}

impl<K: Ord, V> AvlNode<K, V> {
    fn new(key: K, value: V) -> Box<Self> {
        Box::new(AvlNode { key, value, height: 1, left: None, right: None })
    }

    fn update(&mut self) {
        self.height = 1 + max(height(&self.left), height(&self.right));
    }

    fn balance_factor(&self) -> i32 {
        height(&self.left) - height(&self.right)
    }
}

fn rotate_right<K: Ord, V>(mut node: Box<AvlNode<K, V>>) -> Box<AvlNode<K, V>> {
    let mut new_root = node.left.take().expect("rotate_right needs a left child");
    node.left = new_root.right.take();
    node.update();
    new_root.right = Some(node);
    new_root.update();
    new_root
}

fn rotate_left<K: Ord, V>(mut node: Box<AvlNode<K, V>>) -> Box<AvlNode<K, V>> {
    let mut new_root = node.right.take().expect("rotate_left needs a right child");
    node.right = new_root.left.take();
    node.update();
    new_root.left = Some(node);
    new_root.update();
    new_root
}

fn rebalance<K: Ord, V>(mut node: Box<AvlNode<K, V>>) -> Box<AvlNode<K, V>> {
    node.update();
    let bf = node.balance_factor();
    if bf > 1 {
        if node.left.as_ref().unwrap().balance_factor() < 0 {
            node.left = Some(rotate_left(node.left.take().unwrap()));
        }
        return rotate_right(node);
    }
    if bf < -1 {
        if node.right.as_ref().unwrap().balance_factor() > 0 {
            node.right = Some(rotate_right(node.right.take().unwrap()));
        }
        return rotate_left(node);
    }
    node
}

fn insert<K: Ord, V>(link: Link<K, V>, key: K, value: V) -> (Box<AvlNode<K, V>>, Option<V>) {
    match link {
        None => (AvlNode::new(key, value), None),
        Some(mut node) => {
            let old = match key.cmp(&node.key) {
                Ordering::Less => {
                    let (child, old) = insert(node.left.take(), key, value);
                    node.left = Some(child);
                    old
                }
                Ordering::Greater => {
                    let (child, old) = insert(node.right.take(), key, value);
                    node.right = Some(child);
                    old
                }
                Ordering::Equal => Some(std::mem::replace(&mut node.value, value)),
            };
            (rebalance(node), old)
        }
    }
}

fn remove_min<K: Ord, V>(mut node: Box<AvlNode<K, V>>) -> (Link<K, V>, Box<AvlNode<K, V>>) {
    match node.left.take() {
        None => {
            let right = node.right.take();
            (right, node)
        }
        Some(left) => {
            let (new_left, min) = remove_min(left);
            node.left = new_left;
            (Some(rebalance(node)), min)
        }
    }
}

fn remove<K: Ord, V>(link: Link<K, V>, key: &K) -> (Link<K, V>, Option<V>) {
    let mut node = match link {
        None => return (None, None),
        Some(n) => n,
    };
    match key.cmp(&node.key) {
        Ordering::Less => {
            let (l, v) = remove(node.left.take(), key);
            node.left = l;
            (Some(rebalance(node)), v)
        }
        Ordering::Greater => {
            let (r, v) = remove(node.right.take(), key);
            node.right = r;
            (Some(rebalance(node)), v)
        }
        Ordering::Equal => {
            let left = node.left.take();
            let right = node.right.take();
            let value = node.value;
            match (left, right) {
                (None, None) => (None, Some(value)),
                (Some(l), None) => (Some(l), Some(value)),
                (None, Some(r)) => (Some(r), Some(value)),
                (Some(l), Some(r)) => {
                    let (rest, mut succ) = remove_min(r);
                    succ.left = Some(l);
                    succ.right = rest;
                    (Some(rebalance(succ)), Some(value))
                }
            }
        }
    }
}

pub struct AvlMap<K, V> {
    root: Link<K, V>,
    size: usize,
}

impl<K: Ord + Display, V: Display> AvlMap<K, V> {
    pub fn new() -> Self {
        AvlMap { root: None, size: 0 }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let (root, old) = insert(self.root.take(), key, value);
        self.root = Some(root);
        if old.is_none() {
            self.size += 1;
        }
        old
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let (root, v) = remove(self.root.take(), key);
        self.root = root;
        if v.is_some() {
            self.size -= 1;
        }
        v
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let mut cur = self.root.as_ref();
        while let Some(n) = cur {
            cur = match key.cmp(&n.key) {
                Ordering::Less => n.left.as_ref(),
                Ordering::Greater => n.right.as_ref(),
                Ordering::Equal => return Some(&n.value),
            };
        }
        None
    }

    pub fn height(&self) -> i32 {
        height(&self.root)
    }

    pub fn keys(&self) -> Vec<&K> {
        fn walk<'a, K, V>(link: &'a Link<K, V>, out: &mut Vec<&'a K>) {
            if let Some(n) = link {
                walk(&n.left, out);
                out.push(&n.key);
                walk(&n.right, out);
            }
        }
        let mut out = Vec::new();
        walk(&self.root, &mut out);
        out
    }

    pub fn check(&self) -> bool {
        fn ok<K: Ord, V>(link: &Link<K, V>) -> Option<i32> {
            match link {
                None => Some(0),
                Some(n) => {
                    let l = ok(&n.left)?;
                    let r = ok(&n.right)?;
                    if (l - r).abs() > 1 || n.height != 1 + l.max(r) {
                        None
                    } else {
                        Some(n.height)
                    }
                }
            }
        }
        ok(&self.root).is_some()
    }

    pub fn shape(&self) -> String {
        fn go<K: Display, V>(link: &Link<K, V>) -> String {
            match link {
                None => "-".into(),
                Some(n) if n.left.is_none() && n.right.is_none() => n.key.to_string(),
                Some(n) => format!("{}[{},{}]", n.key, go(&n.left), go(&n.right)),
            }
        }
        go(&self.root)
    }
}

fn main() {
    let mut m = AvlMap::new();
    for i in 1..=15 {
        m.insert(i, i * i);
        if i % 5 == 0 {
            println!("after {:2}: height {} balanced {} shape {}", i, m.height(), m.check(), m.shape());
        }
    }
    println!("size {} keys {:?}", m.size, m.keys());
    println!("get 7 = {:?}, get 99 = {:?}", m.get(&7), m.get(&99));
    println!("overwrite 7: {:?} -> {:?}", m.insert(7, -1), m.get(&7));

    for k in [8, 1, 2, 3, 15, 42] {
        let v = m.remove(&k);
        println!("remove {:2} -> {:?}; height {} balanced {} size {}", k, v, m.height(), m.check(), m.size);
    }
    println!("shape {}", m.shape());

    let mut words: AvlMap<String, usize> = AvlMap::new();
    let text = "it was the best of times it was the worst of times";
    for w in text.split(' ') {
        let n = words.get(&w.to_string()).copied().unwrap_or(0);
        words.insert(w.to_string(), n + 1);
    }
    for k in words.keys() {
        print!("{}={} ", k, words.get(k).unwrap());
    }
    println!();
    println!("words height {} balanced {}", words.height(), words.check());

    // pseudo-random workload
    let mut seed: u64 = 12345;
    let mut big: AvlMap<u64, u64> = AvlMap::new();
    for _ in 0..250 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let k = (seed >> 33) % 1000;
        big.insert(k, seed % 7);
    }
    let before = big.size;
    let mut removed = 0;
    for k in (0..1000).step_by(3) {
        if big.remove(&k).is_some() {
            removed += 1;
        }
    }
    println!("big: {} keys, removed {}, now {}, height {}, balanced {}", before, removed, big.size, big.height(), big.check());
}
