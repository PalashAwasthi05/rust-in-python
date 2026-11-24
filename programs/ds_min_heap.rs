// Binary min-heap from scratch, generic over T: Ord, with heapify, sift up/down, and heap sort.
use std::fmt::Debug;

#[derive(Debug, Clone)]
struct MinHeap<T> {
    data: Vec<T>,
}

impl<T: Ord + Debug> MinHeap<T> {
    fn new() -> Self {
        MinHeap { data: Vec::new() }
    }

    fn from_vec(data: Vec<T>) -> Self {
        let mut h = MinHeap { data };
        if h.data.len() > 1 {
            for i in (0..h.data.len() / 2).rev() {
                h.sift_down(i);
            }
        }
        h
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    fn peek(&self) -> Option<&T> {
        self.data.first()
    }

    fn push(&mut self, v: T) {
        self.data.push(v);
        let last = self.data.len() - 1;
        self.sift_up(last);
    }

    fn pop(&mut self) -> Option<T> {
        if self.data.is_empty() {
            return None;
        }
        let last = self.data.len() - 1;
        self.data.swap(0, last);
        let top = self.data.pop();
        if !self.data.is_empty() {
            self.sift_down(0);
        }
        top
    }

    fn sift_up(&mut self, mut i: usize) {
        while i > 0 {
            let parent = (i - 1) / 2;
            if self.data[i] < self.data[parent] {
                self.data.swap(i, parent);
                i = parent;
            } else {
                break;
            }
        }
    }

    fn sift_down(&mut self, mut i: usize) {
        let n = self.data.len();
        loop {
            let l = 2 * i + 1;
            let r = l + 1;
            let mut smallest = i;
            if l < n && self.data[l] < self.data[smallest] {
                smallest = l;
            }
            if r < n && self.data[r] < self.data[smallest] {
                smallest = r;
            }
            if smallest == i {
                return;
            }
            self.data.swap(i, smallest);
            i = smallest;
        }
    }

    fn is_valid(&self) -> bool {
        (1..self.data.len()).all(|i| self.data[(i - 1) / 2] <= self.data[i])
    }

    fn into_sorted(mut self) -> Vec<T> {
        let mut out = Vec::with_capacity(self.len());
        while let Some(x) = self.pop() {
            out.push(x);
        }
        out
    }
}

impl<T: Ord + Debug> Extend<T> for MinHeap<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for x in iter {
            self.push(x);
        }
    }
}

impl<T: Ord + Debug> FromIterator<T> for MinHeap<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        MinHeap::from_vec(iter.into_iter().collect())
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Task {
    priority: u8,
    name: String,
}

fn dijkstra(adj: &Vec<Vec<(usize, u32)>>, src: usize) -> Vec<Option<u32>> {
    let mut dist: Vec<Option<u32>> = vec![None; adj.len()];
    let mut heap = MinHeap::new();
    dist[src] = Some(0);
    heap.push((0u32, src));
    while let Some((d, u)) = heap.pop() {
        if let Some(best) = dist[u] {
            if d > best {
                continue;
            }
        }
        for &(v, w) in &adj[u] {
            let nd = d + w;
            let better = match dist[v] {
                None => true,
                Some(old) => nd < old,
            };
            if better {
                dist[v] = Some(nd);
                heap.push((nd, v));
            }
        }
    }
    dist
}

fn k_smallest_merged(lists: &[Vec<i32>]) -> Vec<i32> {
    let mut heap = MinHeap::new();
    for (li, l) in lists.iter().enumerate() {
        if let Some(&first) = l.first() {
            heap.push((first, li, 0usize));
        }
    }
    let mut out = vec![];
    while let Some((v, li, idx)) = heap.pop() {
        out.push(v);
        if let Some(&next) = lists[li].get(idx + 1) {
            heap.push((next, li, idx + 1));
        }
    }
    out
}

fn main() {
    let mut h = MinHeap::new();
    for x in [5, 3, 8, 1, 9, 2, 7] {
        h.push(x);
        println!("push {} -> top {:?} valid {}", x, h.peek(), h.is_valid());
    }
    println!("raw {:?}", h.data);
    print!("pops:");
    for _ in 0..3 {
        print!(" {:?}", h.pop());
    }
    println!();
    h.extend(vec![0, 4, 6]);
    println!("after extend len {} top {:?}", h.len(), h.peek());
    println!("sorted {:?}", h.clone().into_sorted());

    let heapified = MinHeap::from_vec(vec![10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
    println!("heapified {:?} valid {}", heapified.data, heapified.is_valid());

    let collected: MinHeap<char> = "heapsort".chars().collect();
    let s: String = collected.into_sorted().into_iter().collect();
    println!("chars sorted: {}", s);

    let mut tasks = MinHeap::new();
    tasks.push(Task { priority: 3, name: "write docs".into() });
    tasks.push(Task { priority: 1, name: "fix prod".into() });
    tasks.push(Task { priority: 2, name: "review".into() });
    tasks.push(Task { priority: 1, name: "deploy".into() });
    while let Some(Task { priority, name }) = tasks.pop() {
        println!("p{} {}", priority, name);
    }
    println!("empty {} pop {:?}", tasks.is_empty(), tasks.pop());

    let adj = vec![
        vec![(1, 4), (2, 1)],
        vec![(3, 1)],
        vec![(1, 2), (3, 5)],
        vec![(4, 3)],
        vec![],
        vec![(0, 1)],
    ];
    let d = dijkstra(&adj, 0);
    for (i, x) in d.iter().enumerate() {
        match x {
            Some(v) => println!("dist[{}] = {}", i, v),
            None => println!("dist[{}] = unreachable", i),
        }
    }

    let merged = k_smallest_merged(&[vec![1, 4, 9], vec![2, 3, 10, 11], vec![], vec![0, 5]]);
    println!("merged {:?}", merged);
}
