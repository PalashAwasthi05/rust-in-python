use std::collections::{HashSet, VecDeque};

struct Graph {
    adj: Vec<Vec<usize>>,
}

impl Graph {
    fn new(n: usize) -> Self { Graph { adj: vec![Vec::new(); n] } }

    fn add_edge(&mut self, a: usize, b: usize) {
        self.adj[a].push(b);
        self.adj[b].push(a);
    }

    fn bfs(&self, start: usize) -> (Vec<usize>, Vec<Option<usize>>) {
        let mut dist = vec![None; self.adj.len()];
        let mut order = Vec::new();
        let mut q = VecDeque::new();
        dist[start] = Some(0);
        q.push_back(start);
        while let Some(u) = q.pop_front() {
            order.push(u);
            for &v in &self.adj[u] {
                if dist[v].is_none() {
                    dist[v] = Some(dist[u].unwrap() + 1);
                    q.push_back(v);
                }
            }
        }
        (order, dist)
    }

    fn dfs(&self, u: usize, seen: &mut HashSet<usize>, out: &mut Vec<usize>) {
        if !seen.insert(u) { return; }
        out.push(u);
        for &v in &self.adj[u] {
            self.dfs(v, seen, out);
        }
    }

    fn components(&self) -> Vec<Vec<usize>> {
        let mut seen = HashSet::new();
        let mut comps = Vec::new();
        for u in 0..self.adj.len() {
            if !seen.contains(&u) {
                let mut c = Vec::new();
                self.dfs(u, &mut seen, &mut c);
                c.sort();
                comps.push(c);
            }
        }
        comps
    }
}

fn main() {
    let mut g = Graph::new(9);
    let edges = [(0, 1), (0, 2), (1, 3), (2, 3), (3, 4), (4, 5), (6, 7)];
    for &(a, b) in edges.iter() {
        g.add_edge(a, b);
    }
    let (order, dist) = g.bfs(0);
    println!("bfs order: {:?}", order);
    for (node, d) in dist.iter().enumerate() {
        match d {
            Some(d) => println!("  dist(0 -> {}) = {}", node, d),
            None => println!("  dist(0 -> {}) = unreachable", node),
        }
    }
    let mut seen = HashSet::new();
    let mut dfs_order = Vec::new();
    g.dfs(0, &mut seen, &mut dfs_order);
    println!("dfs order: {:?}", dfs_order);
    let comps = g.components();
    println!("{} components: {:?}", comps.len(), comps);
}
