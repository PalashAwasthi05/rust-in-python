use std::collections::{HashMap, HashSet, BTreeMap, VecDeque, BinaryHeap, BTreeSet};
fn main() {
    let mut m: HashMap<String, i32> = HashMap::new();
    for w in "a b a c b a".split_whitespace() { *m.entry(w.to_string()).or_insert(0) += 1; }
    let mut keys: Vec<_> = m.keys().cloned().collect();
    keys.sort();
    for k in &keys { println!("{} {}", k, m[k]); }
    if let Some(v) = m.get("a") { println!("a={v}"); }
    println!("{} {:?} {:?}", m.len(), m.get("zz"), m.contains_key("b"));
    let mut b = BTreeMap::new();
    b.insert(3, "c"); b.insert(1, "a"); b.insert(2, "b");
    println!("{:?}", b);
    for (k, v) in &b { print!("{k}{v} "); }
    println!();
    let mut s = HashSet::new();
    println!("{} {} {}", s.insert(5), s.insert(5), s.len());
    let t: BTreeSet<char> = "hello".chars().collect();
    println!("{:?}", t);
    let mut q = VecDeque::new(); q.push_back(1); q.push_front(0); q.push_back(2);
    println!("{:?} {:?} {:?} {}", q.pop_front(), q, q.front(), q[1]);
    let mut h = BinaryHeap::new();
    for x in [3, 1, 4, 1, 5, 9, 2, 6] { h.push(x); }
    let mut out = Vec::new();
    while let Some(x) = h.pop() { out.push(x); }
    println!("{:?}", out);
    let counts: BTreeMap<char, usize> = "mississippi".chars().fold(BTreeMap::new(), |mut acc, c| { *acc.entry(c).or_default() += 1; acc });
    println!("{:?}", counts);
    let pairs: HashMap<&str, i32> = vec![("x", 1), ("y", 2)].into_iter().collect();
    let mut ks: Vec<_> = pairs.iter().map(|(k, v)| format!("{k}={v}")).collect();
    ks.sort();
    println!("{}", ks.join(","));
    let mut scores = BTreeMap::new();
    scores.insert("bob", vec![1]);
    scores.entry("bob").or_insert_with(Vec::new).push(2);
    scores.entry("amy").or_insert_with(Vec::new).push(3);
    if let Some(list) = scores.get_mut("amy") { list.push(4); }
    let removed = scores.remove("bob");
    println!("{:?} {:?}", scores, removed);
}
