use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;

#[derive(Debug, PartialEq, Eq)]
struct Task {
    priority: u32,
    seq: u32,
    name: String,
}

// Higher priority first; among equal priorities, the earliest submitted (lowest seq) first.
impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority
            .cmp(&other.priority)
            .then_with(|| other.seq.cmp(&self.seq))
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct State {
    cost: u32,
    node: usize,
}

fn dijkstra(adj: &Vec<Vec<(usize, u32)>>, start: usize) -> Vec<Option<u32>> {
    let mut dist: Vec<Option<u32>> = vec![None; adj.len()];
    let mut heap = BinaryHeap::new();
    dist[start] = Some(0);
    heap.push(Reverse(State { cost: 0, node: start }));
    while let Some(Reverse(State { cost, node })) = heap.pop() {
        if let Some(d) = dist[node] {
            if cost > d {
                continue;
            }
        }
        for &(next, w) in &adj[node] {
            let nc = cost + w;
            let better = match dist[next] {
                None => true,
                Some(old) => nc < old,
            };
            if better {
                dist[next] = Some(nc);
                heap.push(Reverse(State { cost: nc, node: next }));
            }
        }
    }
    dist
}

fn main() {
    let mut queue = BinaryHeap::new();
    let jobs = [(2, "write"), (5, "deploy"), (1, "lint"), (5, "test"), (2, "review"), (9, "hotfix")];
    for (seq, (p, n)) in jobs.iter().enumerate() {
        queue.push(Task { priority: *p, seq: seq as u32, name: n.to_string() });
    }
    println!("peek: {:?}", queue.peek().map(|t| &t.name));
    println!("len: {}", queue.len());
    while let Some(task) = queue.pop() {
        println!("run {} (p{} #{})", task.name, task.priority, task.seq);
    }
    println!("empty: {}", queue.is_empty());

    let mut mins = BinaryHeap::new();
    for x in [7, 3, 9, 1, 4, 1].iter() {
        mins.push(Reverse(*x));
    }
    let mut out = Vec::new();
    while let Some(Reverse(x)) = mins.pop() {
        out.push(x);
    }
    println!("ascending: {:?}", out);

    let maxes: BinaryHeap<i32> = vec![3, 8, -2, 8, 5].into_iter().collect();
    println!("sorted: {:?}", maxes.into_sorted_vec());

    let mut pairs = BinaryHeap::new();
    pairs.push((2, "b"));
    pairs.push((2, "a"));
    pairs.push((3, "z"));
    pairs.push((1, "y"));
    let mut order = Vec::new();
    while let Some(p) = pairs.pop() {
        order.push(p);
    }
    println!("{:?}", order);

    // top-k smallest with a bounded max-heap
    let data = [15, 3, 22, 8, 1, 19, 4, 11];
    let mut top = BinaryHeap::new();
    for &d in data.iter() {
        top.push(d);
        if top.len() > 3 {
            top.pop();
        }
    }
    let mut smallest = top.into_vec();
    smallest.sort();
    println!("3 smallest: {:?}", smallest);

    let adj = vec![
        vec![(1, 4), (2, 1)],
        vec![(3, 1)],
        vec![(1, 2), (3, 5)],
        vec![(4, 3)],
        vec![],
        vec![(0, 1)],
    ];
    let d = dijkstra(&adj, 0);
    println!("{:?}", d);

    println!("Reverse cmp: {:?}", Reverse(1).cmp(&Reverse(2)));
    println!("Reverse lt: {}", Reverse("a") < Reverse("b"));
    println!("Reverse eq: {}", Reverse(5) == Reverse(5));
    println!("Reverse inner: {}", Reverse(42).0);
    let mut rv = vec![Reverse(3), Reverse(10), Reverse(1)];
    rv.sort();
    println!("{:?}", rv);
    let mut tagged = vec![(Reverse(2), "x"), (Reverse(5), "y"), (Reverse(2), "a")];
    tagged.sort();
    println!("{:?}", tagged);
}
