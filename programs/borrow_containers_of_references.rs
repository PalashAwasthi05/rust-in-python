use std::collections::HashMap;
fn main() {
    let words = vec!["a".to_string(), "b".to_string(), "a".to_string()];
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for w in &words { *counts.entry(w.as_str()).or_insert(0) += 1; }
    let key = String::from("a");
    if let Some(c) = counts.get_mut(key.as_str()) { *c += 10; }
    drop(key);
    let mut refs: Vec<&i32> = vec![];
    let nums = vec![1, 2, 3];
    for n in &nums { refs.push(n); }
    let mut more = vec![&nums[0]];
    more.extend(refs.iter().copied());
    let mut sorted: Vec<_> = counts.into_iter().collect(); sorted.sort();
    println!("{:?} {:?} {:?}", sorted, refs, more);
    let mut x = 1; let mut y = 2;
    { let mut ptrs: Vec<&mut i32> = vec![]; ptrs.push(&mut x); ptrs.push(&mut y); for p in ptrs.iter_mut() { **p += 1; } }
    x += 1; println!("{} {}", x, y);
}
