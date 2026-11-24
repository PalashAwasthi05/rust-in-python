// Iterators over a binary tree (stack-based in-order) and a graph (BFS with a queue).
use std::collections::{HashSet, VecDeque};

#[derive(Debug)]
struct Node {
    value: i32,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

#[derive(Default)]
struct Tree {
    root: Option<Box<Node>>,
}

impl Tree {
    fn insert(&mut self, value: i32) {
        let mut slot = &mut self.root;
        while let Some(node) = slot {
            slot = if value < node.value { &mut node.left } else { &mut node.right };
        }
        *slot = Some(Box::new(Node { value, left: None, right: None }));
    }

    fn iter(&self) -> InOrder<'_> {
        let mut it = InOrder { stack: Vec::new() };
        it.push_left(self.root.as_deref());
        it
    }
}

struct InOrder<'a> {
    stack: Vec<&'a Node>,
}

impl<'a> InOrder<'a> {
    fn push_left(&mut self, mut node: Option<&'a Node>) {
        while let Some(n) = node {
            self.stack.push(n);
            node = n.left.as_deref();
        }
    }
}

impl<'a> Iterator for InOrder<'a> {
    type Item = i32;
    fn next(&mut self) -> Option<i32> {
        let node = self.stack.pop()?;
        self.push_left(node.right.as_deref());
        Some(node.value)
    }
}

impl<'a> IntoIterator for &'a Tree {
    type Item = i32;
    type IntoIter = InOrder<'a>;
    fn into_iter(self) -> InOrder<'a> {
        self.iter()
    }
}

impl FromIterator<i32> for Tree {
    fn from_iter<I: IntoIterator<Item = i32>>(iter: I) -> Tree {
        let mut t = Tree::default();
        for v in iter {
            t.insert(v);
        }
        t
    }
}

struct Graph {
    edges: Vec<Vec<usize>>,
}

struct Bfs<'g> {
    graph: &'g Graph,
    queue: VecDeque<(usize, u32)>,
    seen: HashSet<usize>,
}

impl Graph {
    fn bfs(&self, start: usize) -> Bfs<'_> {
        let mut seen = HashSet::new();
        seen.insert(start);
        let mut queue = VecDeque::new();
        queue.push_back((start, 0));
        Bfs { graph: self, queue, seen }
    }
}

impl<'g> Iterator for Bfs<'g> {
    type Item = (usize, u32);
    fn next(&mut self) -> Option<(usize, u32)> {
        let (node, depth) = self.queue.pop_front()?;
        for &next in &self.graph.edges[node] {
            if self.seen.insert(next) {
                self.queue.push_back((next, depth + 1));
            }
        }
        Some((node, depth))
    }
}

fn main() {
    let tree: Tree = vec![50, 30, 70, 20, 40, 60, 80, 35].into_iter().collect();
    let sorted: Vec<i32> = tree.iter().collect();
    println!("in order {:?}", sorted);
    let mut total = 0;
    for v in &tree {
        total += v;
    }
    println!("total {}", total);
    println!("between 30 and 65: {:?}", tree.iter().skip_while(|&v| v < 30).take_while(|&v| v <= 65).collect::<Vec<_>>());
    println!("min {:?} max {:?}", tree.iter().next(), tree.iter().last());
    let (small, big): (Vec<i32>, Vec<i32>) = tree.iter().partition(|&v| v < 50);
    println!("{:?} | {:?}", small, big);

    let g = Graph {
        edges: vec![vec![1, 2], vec![3], vec![3, 4], vec![5], vec![5], vec![], vec![0]],
    };
    for (node, depth) in g.bfs(0) {
        println!("visit {} at depth {}", node, depth);
    }
    let reach: Vec<usize> = g.bfs(6).map(|(n, _)| n).collect();
    println!("from 6: {:?}", reach);
    let deepest = g.bfs(0).max_by_key(|&(_, d)| d);
    println!("deepest {:?}", deepest);
    let at_two: Vec<usize> = g.bfs(0).filter(|&(_, d)| d == 2).map(|(n, _)| n).collect();
    println!("depth 2: {:?}", at_two);
}
