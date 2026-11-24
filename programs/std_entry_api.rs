// Entry API matching, double-ended map iteration, VecDeque search, and palindromes.
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::collections::hash_map::Entry;
fn main() {
    let mut m: HashMap<String, Vec<i32>> = HashMap::new();
    for (k, v) in [("a", 1), ("b", 2), ("a", 3)] {
        match m.entry(k.to_string()) {
            Entry::Occupied(mut o) => o.get_mut().push(v),
            Entry::Vacant(e) => { e.insert(vec![v]); }
        }
    }
    let mut ks: Vec<_> = m.iter().collect();
    ks.sort();
    println!("{:?}", ks);
    if let Entry::Occupied(o) = m.entry("a".into()) {
        let k = o.key().clone();
        println!("{:?} {:?}", k, o.remove_entry());
    }
    println!("{}", m.len());
    let e = m.entry("z".into());
    println!("{:?}", e.key());
    let v = e.or_insert_with(|| vec![0]);
    v.push(9);
    println!("{:?}", m.get("z"));
    let mut bt: BTreeMap<i32, &str> = BTreeMap::new();
    bt.insert(5, "five");
    bt.insert(1, "one");
    let mut it = bt.iter();
    println!("{:?} {:?} {:?}", it.next(), it.next_back(), it.next());
    for (_, v) in bt.iter_mut() {
        *v = "x";
    }
    println!("{:?}", bt);
    bt.retain(|k, _| *k > 2);
    println!("{:?} {:?}", bt, bt.get(&5).copied().unwrap_or("none"));
    let mut dq = VecDeque::from([3, 1, 2]);
    dq.make_contiguous().sort();
    println!("{:?} {:?}", dq, dq.binary_search(&2));
    println!("{:?}", dq.iter().rev().copied().collect::<Vec<_>>());
    dq.insert(1, 10);
    println!("{:?} {:?}", dq.remove(0), dq);
    let mut counter: HashMap<&str, usize> = HashMap::new();
    "a b a c b a".split(' ').for_each(|w| *counter.entry(w).or_insert(0) += 1);
    let mut top: Vec<(&str, usize)> = counter.into_iter().collect();
    top.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    println!("{:?}", top);
    let x: Option<Option<i32>> = Some(None);
    println!("{:?} {:?}", x.flatten(), x.is_some());
    let words = vec!["apple", "bob", "kayak", "level", "rust"];
    let pals: Vec<_> = words.iter().filter(|w| w.chars().eq(w.chars().rev())).collect();
    println!("{:?}", pals);
    let s = String::from("héllo");
    println!("{:?} {:?}", s.chars().rev().collect::<String>(), s.char_indices().map(|(i, _)| i).collect::<Vec<_>>());
    println!("{:?}", s.split_at(3));
    println!("{:?}", s.get(1..2));
    println!("{:?}", s.bytes().len());
}
