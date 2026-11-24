use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct State {
    cost: u32,
    node: usize,
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost).then_with(|| self.node.cmp(&other.node))
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

trait Graph {
    type Node: Copy + Eq + std::hash::Hash + std::fmt::Debug;
    fn edges(&self, n: Self::Node) -> Vec<(Self::Node, u32)>;
    fn index(&self, n: Self::Node) -> usize;
    fn node(&self, i: usize) -> Self::Node;
}

struct Grid {
    cells: Vec<Vec<u32>>,
}

impl Graph for Grid {
    type Node = (usize, usize);
    fn edges(&self, (r, c): (usize, usize)) -> Vec<((usize, usize), u32)> {
        let mut out = Vec::new();
        let h = self.cells.len();
        let w = self.cells[0].len();
        if r > 0 {
            out.push(((r - 1, c), self.cells[r - 1][c]));
        }
        if r + 1 < h {
            out.push(((r + 1, c), self.cells[r + 1][c]));
        }
        if c > 0 {
            out.push(((r, c - 1), self.cells[r][c - 1]));
        }
        if c + 1 < w {
            out.push(((r, c + 1), self.cells[r][c + 1]));
        }
        out
    }
    fn index(&self, (r, c): (usize, usize)) -> usize {
        r * self.cells[0].len() + c
    }
    fn node(&self, i: usize) -> (usize, usize) {
        (i / self.cells[0].len(), i % self.cells[0].len())
    }
}

fn shortest<G: Graph>(g: &G, start: G::Node, goal: G::Node) -> Option<(u32, Vec<G::Node>)> {
    let mut dist: HashMap<G::Node, u32> = HashMap::new();
    let mut prev: HashMap<G::Node, G::Node> = HashMap::new();
    let mut heap = BinaryHeap::new();
    dist.insert(start, 0);
    heap.push(State { cost: 0, node: g.index(start) });
    while let Some(State { cost, node }) = heap.pop() {
        let n = g.node(node);
        if n == goal {
            let mut path = vec![goal];
            let mut cur = goal;
            while let Some(&p) = prev.get(&cur) {
                path.push(p);
                cur = p;
            }
            path.reverse();
            return Some((cost, path));
        }
        if cost > *dist.get(&n).unwrap_or(&u32::MAX) {
            continue;
        }
        for (next, w) in g.edges(n) {
            let nc = cost + w;
            if nc < *dist.get(&next).unwrap_or(&u32::MAX) {
                dist.insert(next, nc);
                prev.insert(next, n);
                heap.push(State { cost: nc, node: g.index(next) });
            }
        }
    }
    None
}

fn main() {
    let input = "1163751\n1381373\n2136511\n3694931\n7463417";
    let cells: Vec<Vec<u32>> = input.lines().map(|l| l.chars().map(|c| c.to_digit(10).unwrap()).collect()).collect();
    let g = Grid { cells };
    let goal = (g.cells.len() - 1, g.cells[0].len() - 1);
    match shortest(&g, (0, 0), goal) {
        Some((cost, path)) => {
            println!("cost {}", cost);
            println!("{} steps: {:?}", path.len(), &path[..4]);
        }
        None => println!("no path"),
    }
    let mut heap: BinaryHeap<State> = vec![State { cost: 5, node: 1 }, State { cost: 2, node: 9 }, State { cost: 2, node: 3 }]
        .into_iter()
        .collect();
    heap.push(State { cost: 7, node: 0 });
    println!("{:?}", heap.peek());
    let order: Vec<(u32, usize)> = std::iter::from_fn(|| heap.pop()).map(|s| (s.cost, s.node)).collect();
    println!("{:?}", order);
    let mut states = vec![State { cost: 3, node: 1 }, State { cost: 1, node: 2 }, State { cost: 3, node: 0 }];
    states.sort();
    println!("{:?}", states);
    println!("{:?}", State { cost: 1, node: 0 }.max(State { cost: 9, node: 0 }));
}
