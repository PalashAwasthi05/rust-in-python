use std::collections::{HashMap, BTreeMap};
fn halves(v: &mut [i32]) -> (&mut [i32], &mut [i32]) { let n = v.len() / 2; v.split_at_mut(n) }
fn both(p: &mut (i32, String)) -> (&mut i32, &mut String) { (&mut p.0, &mut p.1) }
fn main() {
    let mut v = vec![1, 2, 3, 4, 5, 6];
    let (a, b) = v.split_at_mut(3);
    a[0] = b[0];
    b[1] += a[1];
    println!("{:?} {:?}", a, b);
    let (x, y) = halves(&mut v);
    x.swap(0, 1);
    y.reverse();
    v.push(7);
    println!("{:?}", v);
    let mut p = (1, String::from("s"));
    let (n, s) = both(&mut p);
    *n += 1;
    s.push('!');
    println!("{:?}", p);
    let mut m: HashMap<String, i32> = HashMap::new();
    m.insert("a".to_string(), 1);
    if let Some((k, val)) = m.iter_mut().next() { *val += k.len() as i32; }
    for (k, val) in m.iter_mut() { *val *= 2; let _ = k; }
    m.insert("b".to_string(), 3);
    let mut bt = BTreeMap::new();
    bt.insert(1, vec![1]);
    let entry = bt.get_mut(&1);
    match entry { Some(list) => list.push(2), None => {} }
    bt.insert(2, vec![]);
    let (first_k, first_v) = bt.iter().next().unwrap();
    println!("{} {:?} {:?}", first_k, first_v, m.get("a"));
    let mut arr = [1, 2, 3, 4];
    let (l, r) = arr.split_at_mut(2);
    l[0] += r[1];
    let mut chunks = arr.chunks_mut(2);
    let c1 = chunks.next().unwrap();
    c1[0] = 0;
    println!("{:?}", arr);
    let mut words = vec!["a".to_string(), "b".to_string()];
    if let Some((last, rest)) = words.split_last_mut() { last.push_str(&rest[0]); }
    words.push("c".into());
    let mut nums = vec![3, 1, 2];
    let res: Result<&mut i32, String> = nums.iter_mut().max().ok_or("none".to_string());
    if let Ok(m) = res { *m = 0; }
    nums.sort();
    println!("{:?} {:?}", words, nums);
}
