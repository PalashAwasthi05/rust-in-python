// Disjoint-set union with path compression and union by rank; Kruskal's MST on top.

struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<u8>,
    size: Vec<usize>,
    sets: usize,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        UnionFind { parent: (0..n).collect(), rank: vec![0; n], size: vec![1; n], sets: n }
    }

    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            let root = self.find(self.parent[x]);
            self.parent[x] = root;
        }
        self.parent[x]
    }

    fn find_iterative(&mut self, mut x: usize) -> usize {
        let mut root = x;
        while self.parent[root] != root {
            root = self.parent[root];
        }
        while self.parent[x] != root {
            let next = self.parent[x];
            self.parent[x] = root;
            x = next;
        }
        root
    }

    fn union(&mut self, a: usize, b: usize) -> bool {
        let (mut ra, mut rb) = (self.find(a), self.find_iterative(b));
        if ra == rb {
            return false;
        }
        if self.rank[ra] < self.rank[rb] {
            std::mem::swap(&mut ra, &mut rb);
        }
        self.parent[rb] = ra;
        self.size[ra] += self.size[rb];
        if self.rank[ra] == self.rank[rb] {
            self.rank[ra] += 1;
        }
        self.sets -= 1;
        true
    }

    fn connected(&mut self, a: usize, b: usize) -> bool {
        self.find(a) == self.find(b)
    }

    fn set_size(&mut self, x: usize) -> usize {
        let r = self.find(x);
        self.size[r]
    }

    fn groups(&mut self) -> Vec<Vec<usize>> {
        let n = self.parent.len();
        let mut by_root: std::collections::BTreeMap<usize, Vec<usize>> = std::collections::BTreeMap::new();
        for i in 0..n {
            let r = self.find(i);
            by_root.entry(r).or_insert_with(Vec::new).push(i);
        }
        let mut gs: Vec<Vec<usize>> = by_root.into_values().collect();
        gs.sort_by(|a, b| b.len().cmp(&a.len()).then(a[0].cmp(&b[0])));
        gs
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Edge {
    w: u32,
    u: usize,
    v: usize,
}

fn kruskal(n: usize, edges: &[Edge]) -> (u32, Vec<Edge>) {
    let mut sorted = edges.to_vec();
    sorted.sort();
    let mut uf = UnionFind::new(n);
    let mut total = 0;
    let mut chosen = Vec::new();
    for e in sorted {
        if uf.union(e.u, e.v) {
            total += e.w;
            chosen.push(e);
            if chosen.len() == n - 1 {
                break;
            }
        }
    }
    (total, chosen)
}

fn main() {
    let mut uf = UnionFind::new(10);
    let pairs = [(0, 1), (2, 3), (1, 3), (4, 5), (6, 7), (5, 7), (0, 3), (8, 8)];
    for &(a, b) in &pairs {
        let merged = uf.union(a, b);
        println!("union({}, {}) -> {:5} sets={}", a, b, merged, uf.sets);
    }
    println!("connected(0,2)={} connected(0,4)={} connected(4,6)={}", uf.connected(0, 2), uf.connected(0, 4), uf.connected(4, 6));
    println!("size of 3's set: {}, of 9's set: {}", uf.set_size(3), uf.set_size(9));
    println!("groups: {:?}", uf.groups());
    println!("max rank: {}", uf.rank.iter().max().unwrap());

    // chain compression
    let mut chain = UnionFind::new(8);
    for i in 0..7 {
        chain.parent[i] = i + 1;
    }
    println!("before: {:?}", chain.parent);
    chain.find(0);
    println!("after find(0): {:?}", chain.parent);

    let edges = vec![
        Edge { u: 0, v: 1, w: 7 },
        Edge { u: 0, v: 3, w: 5 },
        Edge { u: 1, v: 2, w: 8 },
        Edge { u: 1, v: 3, w: 9 },
        Edge { u: 1, v: 4, w: 7 },
        Edge { u: 2, v: 4, w: 5 },
        Edge { u: 3, v: 4, w: 15 },
        Edge { u: 3, v: 5, w: 6 },
        Edge { u: 4, v: 5, w: 8 },
        Edge { u: 4, v: 6, w: 9 },
        Edge { u: 5, v: 6, w: 11 },
    ];
    let (total, mst) = kruskal(7, &edges);
    println!("MST weight {}", total);
    for e in &mst {
        println!("  {}-{} ({})", e.u, e.v, e.w);
    }

    // count islands in a grid
    let grid = ["11000", "11010", "00100", "00011", "10011"];
    let rows = grid.len();
    let cols = grid[0].len();
    let cells: Vec<Vec<bool>> = grid.iter().map(|r| r.bytes().map(|b| b == b'1').collect()).collect();
    let mut islands = UnionFind::new(rows * cols);
    let mut water = 0;
    for r in 0..rows {
        for c in 0..cols {
            if !cells[r][c] {
                water += 1;
                continue;
            }
            if r + 1 < rows && cells[r + 1][c] {
                islands.union(r * cols + c, (r + 1) * cols + c);
            }
            if c + 1 < cols && cells[r][c + 1] {
                islands.union(r * cols + c, r * cols + c + 1);
            }
        }
    }
    println!("islands: {}", islands.sets - water);
}
