use std::collections::HashMap;

#[derive(Debug)]
struct Node {
    children: Vec<usize>,
    value: i32,
}

struct Tree {
    nodes: Vec<Node>,
}

impl Tree {
    fn sum(&self, i: usize) -> i32 {
        self.nodes[i].value + self.nodes[i].children.iter().map(|&c| self.sum(c)).sum::<i32>()
    }
    fn double_all(&mut self) {
        for n in self.nodes.iter_mut() {
            n.value *= 2;
        }
    }
    fn add_child(&mut self, parent: usize, value: i32) -> usize {
        let id = self.nodes.len();
        self.nodes.push(Node { children: vec![], value });
        self.nodes[parent].children.push(id);
        id
    }
    fn get_or_add(&mut self, parent: usize, value: i32) -> &mut Node {
        let existing = self.nodes[parent].children.iter().copied().find(|&c| self.nodes[c].value == value);
        let id = match existing {
            Some(id) => id,
            None => self.add_child(parent, value),
        };
        &mut self.nodes[id]
    }
}

fn process<F>(items: &mut Vec<String>, mut f: F) -> usize
where
    F: FnMut(&mut String) -> bool,
{
    let mut changed = 0;
    for it in items.iter_mut() {
        if f(it) {
            changed += 1;
        }
    }
    changed
}

fn memo_fib(n: u64, memo: &mut HashMap<u64, u64>) -> u64 {
    if n < 2 {
        return n;
    }
    if let Some(&v) = memo.get(&n) {
        return v;
    }
    let v = memo_fib(n - 1, memo) + memo_fib(n - 2, memo);
    memo.insert(n, v);
    v
}

fn first_word(s: &str) -> &str {
    s.split(' ').next().unwrap_or(s)
}

fn main() {
    let mut t = Tree { nodes: vec![Node { children: vec![], value: 1 }] };
    let a = t.add_child(0, 2);
    t.add_child(a, 3);
    t.add_child(0, 4);
    println!("sum {}", t.sum(0));
    t.double_all();
    println!("sum {}", t.sum(0));
    t.get_or_add(0, 8).value += 100;
    t.get_or_add(0, 50).value += 1;
    println!("{:?}", t.nodes.iter().map(|n| n.value).collect::<Vec<_>>());

    let mut words = vec!["apple".to_string(), "Bob".to_string(), "cat".to_string()];
    let mut log = Vec::new();
    let n = process(&mut words, |w| {
        if w.chars().next().map_or(false, |c| c.is_lowercase()) {
            log.push(w.clone());
            *w = w.to_uppercase();
            true
        } else {
            false
        }
    });
    println!("{} {:?} {:?}", n, words, log);

    let mut memo = HashMap::new();
    println!("fib 50 = {}", memo_fib(50, &mut memo));
    println!("memo size {}", memo.len());

    let mut s = String::from("hello world");
    let w = first_word(&s).to_string();
    s.clear();
    println!("[{}] [{}]", w, s);

    let mut v = vec![1, 2, 3, 4, 5];
    let len = v.len();
    v[len - 1] = v[0] + v[1];
    let idx = v.iter().position(|&x| x == 3).unwrap();
    v.remove(idx);
    v.swap(0, len - 2);
    println!("{:?}", v);

    let mut stack = vec![vec![1], vec![2, 3]];
    while let Some(mut top) = stack.pop() {
        if let Some(x) = top.pop() {
            println!("pop {}", x);
            if !top.is_empty() {
                stack.push(top);
            }
        }
    }

    let mut m: HashMap<&str, Vec<i32>> = HashMap::new();
    m.insert("a", vec![]);
    if let Some(list) = m.get_mut("a") {
        list.push(1);
        list.push(2);
    }
    let total: i32 = m.values().flatten().sum();
    println!("total {}", total);

    let mut counter = 0;
    let mut next_id = || {
        counter += 1;
        format!("id{}", counter)
    };
    let ids: Vec<String> = (0..3).map(|_| next_id()).collect();
    println!("{:?} {}", ids, counter);

    let mut opt: Option<Vec<i32>> = Some(vec![]);
    for i in 0..3 {
        if let Some(v) = opt.as_mut() {
            v.push(i);
        }
    }
    let taken = opt.take().unwrap_or_default();
    println!("{:?} {:?}", taken, opt);

    let mut a = [1, 2, 3];
    let r = &mut a;
    r[0] = 10;
    let r2 = &*r;
    println!("{:?}", r2);
    a[1] = 20;
    println!("{:?}", a);
}
