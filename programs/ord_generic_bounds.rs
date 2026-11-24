use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Pair<T> {
    first: T,
    second: T,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Tree<T> {
    Leaf(T),
    Node(Box<Tree<T>>, Box<Tree<T>>),
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
struct Wrapper<T>(T);

fn largest<T: PartialOrd + Copy>(items: &[T]) -> T {
    let mut best = items[0];
    for &item in items {
        if item > best {
            best = item;
        }
    }
    best
}

fn sorted<T: Ord + Clone>(items: &[T]) -> Vec<T> {
    let mut v = items.to_vec();
    v.sort();
    v
}

fn count<K: Hash + Eq + Ord + Clone + Debug>(items: &[K]) -> Vec<(K, usize)> {
    let mut counts: HashMap<K, usize> = HashMap::new();
    for k in items {
        *counts.entry(k.clone()).or_insert(0) += 1;
    }
    let mut out: Vec<(K, usize)> = counts.into_iter().collect();
    out.sort();
    out
}

fn is_sorted_by_key<T, K: PartialOrd, F: Fn(&T) -> K>(items: &[T], key: F) -> bool {
    items.windows(2).all(|w| key(&w[0]) <= key(&w[1]))
}

fn compare_all<T: Ord>(a: &[T], b: &[T]) -> Ordering {
    for (x, y) in a.iter().zip(b.iter()) {
        match x.cmp(y) {
            Ordering::Equal => continue,
            other => return other,
        }
    }
    a.len().cmp(&b.len())
}

fn leaf<T>(t: T) -> Tree<T> {
    Tree::Leaf(t)
}

fn node<T>(a: Tree<T>, b: Tree<T>) -> Tree<T> {
    Tree::Node(Box::new(a), Box::new(b))
}

fn main() {
    println!("{}", largest(&[3, 9, 2, 7]));
    println!("{}", largest(&[1.5, -0.5, 8.25]));
    println!("{}", largest(&['q', 'z', 'a']));
    println!("{:?}", largest(&[Pair { first: 1, second: 9 }, Pair { first: 2, second: 0 }, Pair { first: 1, second: 10 }]));

    let pairs = vec![
        Pair { first: "b", second: "a" },
        Pair { first: "a", second: "z" },
        Pair { first: "a", second: "c" },
    ];
    println!("{:?}", sorted(&pairs));
    println!("{:?}", count(&["x", "y", "x", "z", "x", "y"]));
    println!("{:?}", count(&[Pair { first: 1, second: 2 }, Pair { first: 1, second: 2 }, Pair { first: 0, second: 5 }]));
    println!("{:?}", count(&[Some(3), None, Some(3), None, None]));

    let trees = vec![
        node(leaf(1), leaf(2)),
        leaf(5),
        node(leaf(1), leaf(1)),
        leaf(-3),
        node(node(leaf(0), leaf(0)), leaf(9)),
    ];
    let mut st = trees.clone();
    st.sort();
    for t in &st {
        println!("{:?}", t);
    }
    println!("leaf < node: {}", leaf(100) < node(leaf(0), leaf(0)));
    println!("tree eq: {}", node(leaf(1), leaf(2)) == node(leaf(1), leaf(2)));
    let mut by_tree: BTreeMap<Tree<i32>, &str> = BTreeMap::new();
    by_tree.insert(leaf(2), "two");
    by_tree.insert(node(leaf(1), leaf(1)), "pair");
    by_tree.insert(leaf(1), "one");
    println!("{:?}", by_tree.values().collect::<Vec<_>>());
    let mut tree_counts: HashMap<Tree<char>, u32> = HashMap::new();
    for t in [leaf('a'), node(leaf('a'), leaf('b')), leaf('a'), node(leaf('a'), leaf('b'))].iter() {
        *tree_counts.entry(t.clone()).or_default() += 1;
    }
    let mut tc: Vec<_> = tree_counts.into_iter().collect();
    tc.sort();
    println!("{:?}", tc);

    println!("{:?}", Wrapper(1.5).partial_cmp(&Wrapper(2.5)));
    println!("{}", Wrapper("abc") < Wrapper("abd"));
    println!("{}", is_sorted_by_key(&[(1, 'c'), (2, 'b'), (3, 'a')], |p| p.0));
    println!("{}", is_sorted_by_key(&[(1, 'c'), (2, 'b'), (3, 'a')], |p| p.1));
    println!("{:?}", compare_all(&[1, 2, 3], &[1, 2]));
    println!("{:?}", compare_all(&["a", "b"], &["a", "c"]));
    println!("{:?}", compare_all::<i32>(&[], &[]));
}
