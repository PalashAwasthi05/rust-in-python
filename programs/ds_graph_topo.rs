// Directed graph as adjacency list: Kahn topological sort, DFS cycle detection, BFS shortest paths.
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Default)]
struct Graph {
    names: Vec<String>,
    index: HashMap<String, usize>,
    adj: Vec<Vec<usize>>,
}

#[derive(Debug, PartialEq)]
enum TopoError {
    Cycle(Vec<String>),
}

#[derive(Clone, Copy, PartialEq)]
enum Color {
    White,
    Gray,
    Black,
}

impl Graph {
    fn node(&mut self, name: &str) -> usize {
        if let Some(&i) = self.index.get(name) {
            return i;
        }
        let i = self.names.len();
        self.names.push(name.to_owned());
        self.index.insert(name.to_owned(), i);
        self.adj.push(Vec::new());
        i
    }

    fn edge(&mut self, from: &str, to: &str) {
        let a = self.node(from);
        let b = self.node(to);
        if !self.adj[a].contains(&b) {
            self.adj[a].push(b);
        }
    }

    fn kahn(&self) -> Result<Vec<&str>, TopoError> {
        let n = self.names.len();
        let mut indeg = vec![0usize; n];
        for outs in &self.adj {
            for &v in outs {
                indeg[v] += 1;
            }
        }
        // pick the alphabetically smallest ready node each time for a deterministic order
        let mut ready: Vec<usize> = (0..n).filter(|&i| indeg[i] == 0).collect();
        let mut order = Vec::with_capacity(n);
        while !ready.is_empty() {
            ready.sort_by(|&a, &b| self.names[b].cmp(&self.names[a]));
            let u = ready.pop().unwrap();
            order.push(self.names[u].as_str());
            for &v in &self.adj[u] {
                indeg[v] -= 1;
                if indeg[v] == 0 {
                    ready.push(v);
                }
            }
        }
        if order.len() == n {
            Ok(order)
        } else {
            Err(TopoError::Cycle(self.find_cycle().unwrap_or_default()))
        }
    }

    fn find_cycle(&self) -> Option<Vec<String>> {
        fn dfs(g: &Graph, u: usize, color: &mut Vec<Color>, stack: &mut Vec<usize>) -> Option<Vec<String>> {
            color[u] = Color::Gray;
            stack.push(u);
            for &v in &g.adj[u] {
                match color[v] {
                    Color::Gray => {
                        let start = stack.iter().position(|&x| x == v).unwrap();
                        let mut cyc: Vec<String> = stack[start..].iter().map(|&i| g.names[i].clone()).collect();
                        cyc.push(g.names[v].clone());
                        return Some(cyc);
                    }
                    Color::White => {
                        if let Some(c) = dfs(g, v, color, stack) {
                            return Some(c);
                        }
                    }
                    Color::Black => {}
                }
            }
            stack.pop();
            color[u] = Color::Black;
            None
        }
        let mut color = vec![Color::White; self.names.len()];
        let mut stack = Vec::new();
        for u in 0..self.names.len() {
            if color[u] == Color::White {
                if let Some(c) = dfs(self, u, &mut color, &mut stack) {
                    return Some(c);
                }
            }
        }
        None
    }

    fn dfs_postorder(&self) -> Vec<&str> {
        let mut seen = HashSet::new();
        let mut out = Vec::new();
        fn visit<'g>(g: &'g Graph, u: usize, seen: &mut HashSet<usize>, out: &mut Vec<&'g str>) {
            if !seen.insert(u) {
                return;
            }
            for &v in &g.adj[u] {
                visit(g, v, seen, out);
            }
            out.push(&g.names[u]);
        }
        for u in 0..self.names.len() {
            visit(self, u, &mut seen, &mut out);
        }
        out.reverse();
        out
    }

    fn bfs_levels(&self, start: &str) -> Vec<(String, usize)> {
        let s = match self.index.get(start) {
            Some(&s) => s,
            None => return vec![],
        };
        let mut dist = vec![usize::MAX; self.names.len()];
        let mut q = VecDeque::new();
        dist[s] = 0;
        q.push_back(s);
        while let Some(u) = q.pop_front() {
            for &v in &self.adj[u] {
                if dist[v] == usize::MAX {
                    dist[v] = dist[u] + 1;
                    q.push_back(v);
                }
            }
        }
        let mut out: Vec<(String, usize)> = dist
            .iter()
            .enumerate()
            .filter(|(_, &d)| d != usize::MAX)
            .map(|(i, &d)| (self.names[i].clone(), d))
            .collect();
        out.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
        out
    }
}

fn main() {
    let mut g = Graph::default();
    let deps = [
        ("std", "core"),
        ("alloc", "core"),
        ("std", "alloc"),
        ("serde", "std"),
        ("app", "serde"),
        ("app", "log"),
        ("log", "std"),
        ("tests", "app"),
    ];
    for (a, b) in deps.iter() {
        // a depends on b, so b must be built first
        g.edge(b, a);
    }
    println!("nodes: {:?}", g.names);
    match g.kahn() {
        Ok(order) => println!("build order: {}", order.join(" -> ")),
        Err(e) => println!("error: {:?}", e),
    }
    println!("dfs order: {:?}", g.dfs_postorder());
    for (name, d) in g.bfs_levels("core") {
        println!("  {:<6} at depth {}", name, d);
    }
    println!("missing start: {:?}", g.bfs_levels("nope"));
    println!("cycle? {:?}", g.find_cycle());

    g.edge("app", "alloc");
    match g.kahn() {
        Ok(order) => println!("build order: {:?}", order),
        Err(TopoError::Cycle(path)) => println!("cycle detected: {}", path.join(" => ")),
    }

    let mut courses = Graph::default();
    let n = 6;
    for i in 0..n {
        courses.node(&format!("c{}", i));
    }
    for (a, b) in [(5, 2), (5, 0), (4, 0), (4, 1), (2, 3), (3, 1)] {
        courses.edge(&format!("c{}", a), &format!("c{}", b));
    }
    let order = courses.kahn().expect("acyclic");
    println!("courses: {:?}", order);
    let total_edges: usize = courses.adj.iter().map(Vec::len).sum();
    println!("{} nodes, {} edges", courses.names.len(), total_edges);
}
