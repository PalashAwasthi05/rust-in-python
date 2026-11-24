#[derive(Debug)]
struct Node {
    value: i32,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

#[derive(Default)]
struct Tree {
    root: Option<Box<Node>>,
    size: usize,
}

fn insert_node(slot: &mut Option<Box<Node>>, value: i32) -> bool {
    match slot {
        None => {
            *slot = Some(Box::new(Node { value, left: None, right: None }));
            true
        }
        Some(node) => {
            if value < node.value {
                insert_node(&mut node.left, value)
            } else if value > node.value {
                insert_node(&mut node.right, value)
            } else {
                false
            }
        }
    }
}

fn in_order(node: &Option<Box<Node>>, out: &mut Vec<i32>) {
    if let Some(n) = node {
        in_order(&n.left, out);
        out.push(n.value);
        in_order(&n.right, out);
    }
}

fn pre_order(node: &Option<Box<Node>>, out: &mut Vec<i32>) {
    if let Some(n) = node {
        out.push(n.value);
        pre_order(&n.left, out);
        pre_order(&n.right, out);
    }
}

fn height(node: &Option<Box<Node>>) -> usize {
    match node {
        None => 0,
        Some(n) => 1 + height(&n.left).max(height(&n.right)),
    }
}

impl Tree {
    fn insert(&mut self, v: i32) {
        if insert_node(&mut self.root, v) {
            self.size += 1;
        }
    }
    fn contains(&self, v: i32) -> bool {
        let mut cur = &self.root;
        while let Some(n) = cur {
            if v == n.value { return true; }
            cur = if v < n.value { &n.left } else { &n.right };
        }
        false
    }
}

fn main() {
    let mut tree = Tree::default();
    for v in [50, 30, 70, 20, 40, 60, 80, 30, 65] {
        tree.insert(v);
    }
    let mut ino = Vec::new();
    in_order(&tree.root, &mut ino);
    let mut pre = Vec::new();
    pre_order(&tree.root, &mut pre);
    println!("size: {}", tree.size);
    println!("in-order: {:?}", ino);
    println!("pre-order: {:?}", pre);
    println!("height: {}", height(&tree.root));
    for q in [40, 45, 65] {
        println!("contains {}? {}", q, tree.contains(q));
    }
}
