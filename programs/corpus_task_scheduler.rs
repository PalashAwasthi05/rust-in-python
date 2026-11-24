use std::collections::{BTreeMap, BinaryHeap, VecDeque};
use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Task {
    id: u32,
    priority: u8,
    name: String,
    deps: Vec<u32>,
    cost: u32,
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority).then_with(|| other.id.cmp(&self.id))
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

fn topo_order(tasks: &[Task]) -> Result<Vec<u32>, String> {
    let mut indeg: BTreeMap<u32, usize> = tasks.iter().map(|t| (t.id, t.deps.len())).collect();
    let mut users: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for t in tasks {
        for &d in &t.deps { users.entry(d).or_default().push(t.id); }
    }
    let mut ready: VecDeque<u32> = indeg.iter().filter(|(_, &n)| n == 0).map(|(&id, _)| id).collect();
    let mut order = Vec::new();
    while let Some(id) = ready.pop_front() {
        order.push(id);
        for &u in users.get(&id).map(|v| v.as_slice()).unwrap_or(&[]) {
            let n = indeg.get_mut(&u).unwrap();
            *n -= 1;
            if *n == 0 { ready.push_back(u); }
        }
    }
    if order.len() == tasks.len() { Ok(order) } else { Err(format!("cycle detected after {:?}", order)) }
}

fn main() {
    let mk = |id, priority, name: &str, deps: &[u32], cost| Task { id, priority, name: name.to_string(), deps: deps.to_vec(), cost };
    let tasks = vec![
        mk(1, 2, "fetch", &[], 3),
        mk(2, 5, "parse", &[1], 2),
        mk(3, 1, "lint", &[1], 1),
        mk(4, 5, "compile", &[2], 6),
        mk(5, 3, "test", &[4, 3], 4),
        mk(6, 5, "docs", &[2], 2),
    ];
    let mut heap: BinaryHeap<Task> = tasks.iter().cloned().collect();
    print!("by priority:");
    while let Some(t) = heap.pop() { print!(" {}(p{})", t.name, t.priority); }
    println!();

    match topo_order(&tasks) {
        Ok(order) => {
            let names: Vec<&str> = order.iter().map(|id| tasks.iter().find(|t| t.id == *id).unwrap().name.as_str()).collect();
            println!("topo order: {}", names.join(" -> "));
            let mut finish: BTreeMap<u32, u32> = BTreeMap::new();
            for id in &order {
                let t = tasks.iter().find(|t| t.id == *id).unwrap();
                let start = t.deps.iter().map(|d| finish[d]).max().unwrap_or(0);
                finish.insert(t.id, start + t.cost);
                println!("  {:<8} start {:>2} end {:>2}", t.name, start, start + t.cost);
            }
            println!("makespan: {}", finish.values().max().unwrap());
        }
        Err(e) => println!("error: {}", e),
    }
    let mut cyclic = tasks.clone();
    cyclic[0].deps.push(5);
    println!("{:?}", topo_order(&cyclic));
}
