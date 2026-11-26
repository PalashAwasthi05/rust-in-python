use std::collections::HashMap;
struct Node { val: i32, next: Option<Box<Node>> }
fn first_even(v: &Vec<i32>) -> Option<&i32> { let o = v.iter().find(|x| **x % 2 == 0); o }
fn pick<'a>(a: &'a [i32], b: &'a [i32]) -> (Option<&'a i32>, Option<&'a i32>) { (a.first(), b.last()) }
fn main() {
    let mut v = vec![1, 2, 3];
    let o = v.first_mut();
    if let Some(x) = o { *x += 10; }
    v.push(4);
    let o2 = v.last();
    let n = o2.copied().unwrap_or(0);
    v.push(n);
    let mut m: HashMap<&str, i32> = HashMap::new();
    m.insert("a", 1);
    let e = m.get("a");
    match e { Some(x) => println!("{}", x), None => {} }
    m.insert("b", 2);
    let r = m.get_mut("a");
    if let Some(x) = r { *x += 1; }
    m.insert("c", 3);
    let list = Node { val: 1, next: Some(Box::new(Node { val: 2, next: Some(Box::new(Node { val: 3, next: None })) })) };
    let mut cur = Some(&list);
    let mut total = 0;
    while let Some(node) = cur { total += node.val; cur = node.next.as_deref(); }
    println!("{} {:?} {:?}", total, v, first_even(&v));
}
