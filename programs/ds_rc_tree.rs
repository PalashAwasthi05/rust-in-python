// Tree with Rc<RefCell<Node>> children and Weak parent links, as in the Rust book, plus subtree operations.
use std::cell::RefCell;
use std::rc::{Rc, Weak};

#[derive(Debug)]
struct TreeNode {
    name: String,
    value: i32,
    parent: RefCell<Weak<TreeNode>>,
    children: RefCell<Vec<Rc<TreeNode>>>,
}

impl TreeNode {
    fn new(name: &str, value: i32) -> Rc<TreeNode> {
        Rc::new(TreeNode {
            name: name.to_string(),
            value,
            parent: RefCell::new(Weak::new()),
            children: RefCell::new(vec![]),
        })
    }
}

fn add_child(parent: &Rc<TreeNode>, child: Rc<TreeNode>) {
    *child.parent.borrow_mut() = Rc::downgrade(parent);
    parent.children.borrow_mut().push(child);
}

fn detach(node: &Rc<TreeNode>) -> bool {
    let parent = match node.parent.borrow().upgrade() {
        Some(p) => p,
        None => return false,
    };
    parent.children.borrow_mut().retain(|c| !Rc::ptr_eq(c, node));
    *node.parent.borrow_mut() = Weak::new();
    true
}

fn path(node: &Rc<TreeNode>) -> String {
    let mut parts = vec![node.name.clone()];
    let mut cur = node.parent.borrow().upgrade();
    while let Some(p) = cur {
        parts.push(p.name.clone());
        cur = p.parent.borrow().upgrade();
    }
    parts.reverse();
    parts.join("/")
}

fn subtree_sum(node: &Rc<TreeNode>) -> i32 {
    node.value + node.children.borrow().iter().map(subtree_sum).sum::<i32>()
}

fn render(node: &Rc<TreeNode>, depth: usize, out: &mut Vec<String>) {
    out.push(format!("{}{} ({})", "  ".repeat(depth), node.name, node.value));
    for child in node.children.borrow().iter() {
        render(child, depth + 1, out);
    }
}

fn find(node: &Rc<TreeNode>, name: &str) -> Option<Rc<TreeNode>> {
    if node.name == name {
        return Some(Rc::clone(node));
    }
    node.children.borrow().iter().find_map(|c| find(c, name))
}

fn main() {
    let leaf = TreeNode::new("leaf", 3);
    println!("leaf parent = {:?}", leaf.parent.borrow().upgrade().map(|p| p.name.clone()));
    {
        let branch = TreeNode::new("branch", 5);
        add_child(&branch, Rc::clone(&leaf));
        println!("leaf parent = {:?}", leaf.parent.borrow().upgrade().map(|p| p.name.clone()));
        println!(
            "branch strong = {}, weak = {}; leaf strong = {}, weak = {}",
            Rc::strong_count(&branch),
            Rc::weak_count(&branch),
            Rc::strong_count(&leaf),
            Rc::weak_count(&leaf)
        );
    }
    println!("after branch scope: leaf parent = {:?}, strong = {}", leaf.parent.borrow().upgrade().map(|p| p.value), Rc::strong_count(&leaf));

    let root = TreeNode::new("root", 1);
    let a = TreeNode::new("a", 10);
    let b = TreeNode::new("b", 20);
    let a1 = TreeNode::new("a1", 100);
    let a2 = TreeNode::new("a2", 200);
    let b1 = TreeNode::new("b1", 300);
    add_child(&root, Rc::clone(&a));
    add_child(&root, Rc::clone(&b));
    add_child(&a, Rc::clone(&a1));
    add_child(&a, a2);
    add_child(&b, b1);
    add_child(&a1, Rc::clone(&leaf));

    let mut lines = Vec::new();
    render(&root, 0, &mut lines);
    println!("{}", lines.join("\n"));
    println!("sum(root) = {} sum(a) = {}", subtree_sum(&root), subtree_sum(&a));
    println!("path(leaf) = {}", path(&leaf));
    match find(&root, "b1") {
        Some(n) => println!("found b1 under {}", path(&n)),
        None => println!("no b1"),
    }
    println!("find zz: {:?}", find(&root, "zz").map(|n| n.value));

    println!("detach a: {}", detach(&a));
    println!("detach a again: {}", detach(&a));
    println!("path(leaf) after detach = {}", path(&leaf));
    println!("sum(root) = {}", subtree_sum(&root));
    add_child(&b, Rc::clone(&a));
    println!("path(leaf) after move = {}", path(&leaf));
    let mut lines = Vec::new();
    render(&root, 0, &mut lines);
    for l in &lines {
        println!("| {}", l);
    }
    println!("a strong = {} weak = {}", Rc::strong_count(&a), Rc::weak_count(&a));
    let names: Vec<String> = b.children.borrow().iter().map(|c| c.name.to_uppercase()).collect();
    println!("b's children: {:?}", names);
    drop(root);
    println!("after dropping root: b parent alive = {}", b.parent.borrow().upgrade().is_some());
}
