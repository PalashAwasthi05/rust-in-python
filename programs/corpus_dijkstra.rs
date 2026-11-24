use std::cmp::Reverse;
use std::collections::{BinaryHeap, BTreeMap};

type Graph = BTreeMap<&'static str, Vec<(&'static str, u32)>>;

fn build() -> Graph {
    let roads = [
        ("Amber", "Birch", 7), ("Amber", "Cedar", 9), ("Amber", "Fir", 14),
        ("Birch", "Cedar", 10), ("Birch", "Dale", 15), ("Cedar", "Dale", 11),
        ("Cedar", "Fir", 2), ("Dale", "Elm", 6), ("Elm", "Fir", 9),
    ];
    let mut g: Graph = BTreeMap::new();
    for (a, b, w) in roads {
        g.entry(a).or_default().push((b, w));
        g.entry(b).or_default().push((a, w));
    }
    g.entry("Ghost").or_default();
    g
}

fn dijkstra(g: &Graph, src: &'static str) -> (BTreeMap<&'static str, u32>, BTreeMap<&'static str, &'static str>) {
    let mut dist: BTreeMap<&str, u32> = BTreeMap::new();
    let mut prev = BTreeMap::new();
    let mut heap = BinaryHeap::new();
    dist.insert(src, 0);
    heap.push(Reverse((0u32, src)));
    while let Some(Reverse((d, u))) = heap.pop() {
        if d > *dist.get(u).unwrap_or(&u32::MAX) { continue; }
        for &(v, w) in &g[u] {
            let nd = d + w;
            if nd < *dist.get(v).unwrap_or(&u32::MAX) {
                dist.insert(v, nd);
                prev.insert(v, u);
                heap.push(Reverse((nd, v)));
            }
        }
    }
    (dist, prev)
}

fn path(prev: &BTreeMap<&'static str, &'static str>, mut to: &'static str) -> Vec<&'static str> {
    let mut p = vec![to];
    while let Some(&u) = prev.get(to) {
        p.push(u);
        to = u;
    }
    p.reverse();
    p
}

fn main() {
    let g = build();
    let (dist, prev) = dijkstra(&g, "Amber");
    for node in g.keys() {
        match dist.get(node) {
            Some(d) => println!("{:<6} {:>3}  via {}", node, d, path(&prev, node).join(" -> ")),
            None => println!("{:<6} unreachable", node),
        }
    }
    let far = dist.iter().max_by_key(|(_, &d)| d).unwrap();
    println!("farthest reachable: {} at {}", far.0, far.1);
}
