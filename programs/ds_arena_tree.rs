// Arena-allocated n-ary tree: nodes live in a Vec and refer to each other by index.
use std::fmt::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct NodeId(usize);

#[derive(Debug)]
struct Node<T> {
    value: T,
    parent: Option<NodeId>,
    children: Vec<NodeId>,
}

#[derive(Debug, Default)]
struct Arena<T> {
    nodes: Vec<Node<T>>,
}

impl<T> Arena<T> {
    fn new() -> Self {
        Arena { nodes: Vec::new() }
    }

    fn add(&mut self, value: T, parent: Option<NodeId>) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(Node { value, parent, children: vec![] });
        if let Some(p) = parent {
            self.nodes[p.0].children.push(id);
        }
        id
    }

    fn get(&self, id: NodeId) -> &T {
        &self.nodes[id.0].value
    }

    fn get_mut(&mut self, id: NodeId) -> &mut T {
        &mut self.nodes[id.0].value
    }

    fn depth(&self, mut id: NodeId) -> usize {
        let mut d = 0;
        while let Some(p) = self.nodes[id.0].parent {
            d += 1;
            id = p;
        }
        d
    }

    fn path_to_root(&self, id: NodeId) -> Vec<NodeId> {
        let mut path = vec![id];
        let mut cur = id;
        while let Some(p) = self.nodes[cur.0].parent {
            path.push(p);
            cur = p;
        }
        path
    }

    fn lca(&self, a: NodeId, b: NodeId) -> NodeId {
        let pa = self.path_to_root(a);
        let pb = self.path_to_root(b);
        let mut result = *pa.last().unwrap();
        for (x, y) in pa.iter().rev().zip(pb.iter().rev()) {
            if x == y {
                result = *x;
            } else {
                break;
            }
        }
        result
    }

    fn descendants(&self, root: NodeId) -> Vec<NodeId> {
        let mut out = Vec::new();
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            out.push(id);
            for &c in self.nodes[id.0].children.iter().rev() {
                stack.push(c);
            }
        }
        out
    }

    fn reparent(&mut self, id: NodeId, new_parent: NodeId) {
        if let Some(old) = self.nodes[id.0].parent {
            self.nodes[old.0].children.retain(|&c| c != id);
        }
        self.nodes[id.0].parent = Some(new_parent);
        self.nodes[new_parent.0].children.push(id);
    }

    fn fold<A>(&self, id: NodeId, f: &dyn Fn(&T, Vec<A>) -> A) -> A {
        let kids: Vec<A> = self.nodes[id.0].children.iter().map(|&c| self.fold(c, f)).collect();
        f(&self.nodes[id.0].value, kids)
    }
}

impl<T: std::fmt::Display> Arena<T> {
    fn render(&self, root: NodeId) -> String {
        let mut s = String::new();
        self.render_into(root, "", true, true, &mut s);
        s
    }

    fn render_into(&self, id: NodeId, prefix: &str, last: bool, is_root: bool, out: &mut String) {
        if is_root {
            writeln!(out, "{}", self.get(id)).unwrap();
        } else {
            writeln!(out, "{}{}{}", prefix, if last { "`-- " } else { "|-- " }, self.get(id)).unwrap();
        }
        let child_prefix = if is_root {
            String::new()
        } else {
            format!("{}{}", prefix, if last { "    " } else { "|   " })
        };
        let kids = &self.nodes[id.0].children;
        for (i, &c) in kids.iter().enumerate() {
            self.render_into(c, &child_prefix, i + 1 == kids.len(), false, out);
        }
    }
}

#[derive(Debug)]
struct Entry {
    name: &'static str,
    size: u64,
}

impl std::fmt::Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.size == 0 {
            write!(f, "{}/", self.name)
        } else {
            write!(f, "{} ({}b)", self.name, self.size)
        }
    }
}

fn main() {
    let mut fs: Arena<Entry> = Arena::new();
    let root = fs.add(Entry { name: "root", size: 0 }, None);
    let src = fs.add(Entry { name: "src", size: 0 }, Some(root));
    let docs = fs.add(Entry { name: "docs", size: 0 }, Some(root));
    let main_rs = fs.add(Entry { name: "main.rs", size: 1200 }, Some(src));
    let lib = fs.add(Entry { name: "lib", size: 0 }, Some(src));
    let parse = fs.add(Entry { name: "parse.rs", size: 3400 }, Some(lib));
    let eval = fs.add(Entry { name: "eval.rs", size: 5100 }, Some(lib));
    let readme = fs.add(Entry { name: "README.md", size: 800 }, Some(docs));
    fs.add(Entry { name: "Cargo.toml", size: 150 }, Some(root));

    print!("{}", fs.render(root));
    let total = fs.fold(root, &|e: &Entry, kids: Vec<u64>| e.size + kids.iter().sum::<u64>());
    println!("total size {}", total);
    let deepest = fs.fold(root, &|_e, kids: Vec<usize>| 1 + kids.into_iter().max().unwrap_or(0));
    println!("height {}", deepest);
    println!("depth(eval.rs) = {}", fs.depth(eval));
    let path: Vec<&str> = fs.path_to_root(parse).iter().rev().map(|&id| fs.get(id).name).collect();
    println!("path {}", path.join("/"));
    println!("lca(parse, eval) = {}", fs.get(fs.lca(parse, eval)));
    println!("lca(main.rs, eval) = {}", fs.get(fs.lca(main_rs, eval)));
    println!("lca(readme, parse) = {}", fs.get(fs.lca(readme, parse)));

    fs.get_mut(main_rs).size += 300;
    fs.reparent(lib, docs);
    println!("after moving lib under docs:");
    print!("{}", fs.render(root));
    let under_docs: Vec<String> = fs.descendants(docs).into_iter().map(|id| fs.get(id).name.to_string()).collect();
    println!("descendants of docs: {:?}", under_docs);
    let sizes = fs.fold(docs, &|e: &Entry, kids: Vec<u64>| e.size + kids.into_iter().sum::<u64>());
    println!("docs size {}", sizes);

    let mut ids: Vec<NodeId> = fs.descendants(root);
    ids.sort_by_key(|&id| std::cmp::Reverse(fs.get(id).size));
    let top: Vec<_> = ids.iter().take(3).map(|&id| (id, fs.get(id).name)).collect();
    println!("largest: {:?}", top);

    let mut nums: Arena<i32> = Arena::new();
    let r = nums.add(1, None);
    for i in 2..=7 {
        let parent = NodeId((i - 2) / 2);
        nums.add(i as i32, Some(parent));
    }
    print!("{}", nums.render(r));
    let product = nums.fold(r, &|v: &i32, kids: Vec<i64>| *v as i64 * kids.iter().product::<i64>());
    println!("product {}", product);
}
