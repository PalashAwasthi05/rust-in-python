// Event scheduling simulation with BinaryHeap<Reverse<..>>, VecDeque and Ordering impls.
use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashMap, VecDeque};

#[derive(Debug, PartialEq, Eq)]
struct Job {
    priority: u8,
    id: u32,
    cost: u32,
}

impl Ord for Job {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority).then_with(|| other.id.cmp(&self.id))
    }
}

impl PartialOrd for Job {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn dijkstra(adj: &HashMap<u32, Vec<(u32, u32)>>, start: u32, n: usize) -> Vec<Option<u32>> {
    let mut dist: Vec<Option<u32>> = vec![None; n];
    let mut heap = BinaryHeap::new();
    dist[start as usize] = Some(0);
    heap.push(Reverse((0u32, start)));
    while let Some(Reverse((d, node))) = heap.pop() {
        if dist[node as usize].map_or(false, |best| d > best) {
            continue;
        }
        for &(next, w) in adj.get(&node).map(|v| v.as_slice()).unwrap_or(&[]) {
            let nd = d + w;
            if dist[next as usize].map_or(true, |cur| nd < cur) {
                dist[next as usize] = Some(nd);
                heap.push(Reverse((nd, next)));
            }
        }
    }
    dist
}

fn main() {
    let mut queue = BinaryHeap::new();
    for (i, (p, c)) in [(2, 5), (9, 1), (2, 3), (5, 8), (9, 2)].iter().enumerate() {
        queue.push(Job { priority: *p, id: i as u32, cost: *c });
    }
    println!("next up: {:?}", queue.peek());
    let mut clock = 0;
    while let Some(job) = queue.pop() {
        clock += job.cost;
        println!("t={:>3} ran job {} (p{})", clock, job.id, job.priority);
    }

    let mut round_robin: VecDeque<(char, u32)> = VecDeque::from(vec![('a', 5), ('b', 2), ('c', 4)]);
    let quantum = 2;
    let mut order = String::new();
    while let Some((name, left)) = round_robin.pop_front() {
        order.push(name);
        if left > quantum {
            round_robin.push_back((name, left - quantum));
        }
    }
    println!("round robin order: {}", order);

    let mut adj: HashMap<u32, Vec<(u32, u32)>> = HashMap::new();
    for &(a, b, w) in &[(0, 1, 4), (0, 2, 1), (2, 1, 2), (1, 3, 1), (2, 3, 5)] {
        adj.entry(a).or_default().push((b, w));
    }
    let dist = dijkstra(&adj, 0, 5);
    for (node, d) in dist.iter().enumerate() {
        match d {
            Some(d) => println!("node {} at distance {}", node, d),
            None => println!("node {} unreachable", node),
        }
    }

    let mut min_heap: BinaryHeap<Reverse<i64>> = vec![5, -3, 12, 0].into_iter().map(Reverse).collect();
    min_heap.push(Reverse(-10));
    let drained: Vec<i64> = std::iter::from_fn(|| min_heap.pop().map(|Reverse(x)| x)).collect();
    println!("ascending {:?}", drained);

    let mut words: BinaryHeap<(usize, &str)> = "the quick brown fox jumped".split(' ').map(|w| (w.len(), w)).collect();
    let top2: Vec<_> = (0..2).filter_map(|_| words.pop()).collect();
    println!("longest two {:?}, remaining {}", top2, words.len());
    let mut ordered: Vec<_> = words.into_sorted_vec();
    ordered.reverse();
    println!("rest desc {:?}", ordered);
    println!("cmp {:?} {:?} {:?}", 1.cmp(&2), "b".cmp("a"), (1, 'z').cmp(&(1, 'z')));
    println!("max/min {:?} {:?}", std::cmp::max("pear", "apple"), std::cmp::min(Reverse(3), Reverse(7)));
    println!("ordering ops {:?} {:?} {}", Ordering::Less.reverse(), Ordering::Equal.then(Ordering::Greater), Ordering::Less.is_lt());
    let mut deque: VecDeque<i32> = (1..=6).collect();
    deque.rotate_right(2);
    let (front, back) = deque.as_slices();
    println!("deque {:?} total len {}", deque, front.len() + back.len());
    deque.retain(|x| x % 2 == 0);
    deque.truncate(2);
    println!("retained {:?} {:?}", deque, deque.iter().map(|x| x * 10).collect::<Vec<_>>());
    let mut d2: VecDeque<&str> = VecDeque::new();
    d2.extend(["x", "y", "z"]);
    d2.swap(0, 2);
    println!("{:?} {:?} {:?}", d2, d2.get(1), d2.iter().position(|&s| s == "x"));
    let v: Vec<&str> = d2.drain(..).collect();
    println!("drained {:?} now empty {}", v, d2.is_empty());
}
