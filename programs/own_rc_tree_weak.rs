// A tree with Rc children and Weak parent pointers.
use std::cell::RefCell;
use std::rc::{Rc, Weak};

#[derive(Debug)]
struct TreeNode {
    name: String,
    parent: RefCell<Weak<TreeNode>>,
    children: RefCell<Vec<Rc<TreeNode>>>,
}

impl TreeNode {
    fn new(name: &str) -> Rc<TreeNode> {
        Rc::new(TreeNode {
            name: name.to_string(),
            parent: RefCell::new(Weak::new()),
            children: RefCell::new(Vec::new()),
        })
    }

    fn add_child(parent: &Rc<TreeNode>, child: Rc<TreeNode>) {
        *child.parent.borrow_mut() = Rc::downgrade(parent);
        parent.children.borrow_mut().push(child);
    }

    fn path(&self) -> String {
        match self.parent.borrow().upgrade() {
            Some(p) => format!("{}/{}", p.path(), self.name),
            None => self.name.clone(),
        }
    }

    fn count(&self) -> usize {
        1 + self.children.borrow().iter().map(|c| c.count()).sum::<usize>()
    }
}

impl Drop for TreeNode {
    fn drop(&mut self) {
        println!("drop {}", self.name);
    }
}

fn main() {
    let root = TreeNode::new("root");
    let usr = TreeNode::new("usr");
    let bin = TreeNode::new("bin");
    let local = TreeNode::new("local");
    TreeNode::add_child(&root, usr.clone());
    TreeNode::add_child(&root, bin.clone());
    TreeNode::add_child(&usr, local.clone());

    println!("{}", local.path());
    println!("{}", bin.path());
    println!("nodes: {}", root.count());
    println!(
        "usr strong={} weak={}",
        Rc::strong_count(&usr),
        Rc::weak_count(&usr)
    );
    println!(
        "root strong={} weak={}",
        Rc::strong_count(&root),
        Rc::weak_count(&root)
    );

    let orphan_parent;
    {
        let tmp = TreeNode::new("tmp");
        let leaf = TreeNode::new("leaf");
        TreeNode::add_child(&tmp, leaf.clone());
        orphan_parent = leaf;
        println!("inside: {}", orphan_parent.path());
    }
    println!("after: {}", orphan_parent.path());
    println!("parent alive: {}", orphan_parent.parent.borrow().upgrade().is_some());

    let names: Vec<String> = root.children.borrow().iter().map(|c| c.name.clone()).collect();
    println!("{:?}", names);
    drop(usr);
    drop(local);
    println!("dropped handles");
    root.children.borrow_mut().remove(0);
    println!("removed usr subtree");
    drop(bin);
    println!("end of main");
}
