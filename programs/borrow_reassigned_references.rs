struct Node { val: i32, next: Option<Box<Node>> }
fn last_mut(mut cur: &mut Node) -> &mut Node { while cur.next.is_some() { cur = cur.next.as_mut().unwrap(); } cur }
fn main() {
    let mut x = 1; let y = 2;
    let mut r = &x; println!("{}", r);
    r = &y; x += 1; println!("{} {}", r, x);
    let mut v = vec![1, 2, 3]; let w = vec![9];
    let mut o = v.first(); println!("{:?}", o);
    o = w.last(); v.push(4); println!("{:?} {:?}", o, v);
    let mut best: Option<&i32> = None;
    for item in v.iter() { if best.map_or(true, |b| item > b) { best = Some(item); } }
    println!("{:?}", best);
    let mut list = Node { val: 0, next: Some(Box::new(Node { val: 1, next: None })) };
    last_mut(&mut list).next = Some(Box::new(Node { val: 2, next: None }));
    let mut cur = Some(&list); let mut s = 0;
    while let Some(n) = cur { s += n.val; cur = n.next.as_deref(); }
    list.val = 10;
    println!("{} {}", s, list.val);
    let mut words = vec!["a".to_string(), "bb".to_string()];
    let mut longest = &words[0];
    for w in &words { if w.len() > longest.len() { longest = w; } }
    println!("{}", longest);
    words.push("c".into());
    let mut m = 0; let mut mr = &mut m; *mr += 1; let mut k = 5; mr = &mut k; *mr += 1; m += 1;
    println!("{} {}", m, k);
}
