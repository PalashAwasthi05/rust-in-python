// HashSet/BTreeSet algebra, VecDeque, and BinaryHeap with Reverse.
use std::cmp::Reverse;
use std::collections::{BTreeSet, BinaryHeap, HashSet, VecDeque};

fn sorted<T: Ord + Clone>(it: impl Iterator<Item = T>) -> Vec<T> {
    let mut v: Vec<T> = it.collect();
    v.sort();
    v
}

fn main() {
    let a: HashSet<i32> = [1, 2, 3, 4, 5].into_iter().collect();
    let b: HashSet<i32> = HashSet::from([4, 5, 6, 7]);
    println!("union {:?}", sorted(a.union(&b).copied()));
    println!("intersection {:?}", sorted(a.intersection(&b).cloned()));
    println!("difference {:?}", sorted(a.difference(&b).copied()));
    println!("symmetric {:?}", sorted(a.symmetric_difference(&b).copied()));
    let small: HashSet<i32> = [2, 3].iter().cloned().collect();
    println!("subset {} superset {} disjoint {}", small.is_subset(&a), a.is_superset(&small), small.is_disjoint(&b));
    let ops = &a | &b;
    let common = &a & &b;
    let minus = &a - &b;
    let xor = &a ^ &b;
    println!("ops {:?} {:?} {:?} {:?}", sorted(ops.into_iter()), sorted(common.into_iter()), sorted(minus.into_iter()), sorted(xor.into_iter()));
    let mut seen = HashSet::new();
    let dupes: Vec<char> = "programming".chars().filter(|c| !seen.insert(*c)).collect();
    println!("dupes {:?}", dupes);
    let bs: BTreeSet<&str> = ["pear", "apple", "fig", "apple"].into_iter().collect();
    println!("btreeset {:?} first {:?} last {:?}", bs, bs.first(), bs.last());
    println!("range {:?}", bs.range("b"..).collect::<Vec<_>>());
    let other: BTreeSet<&str> = ["fig", "kiwi"].into_iter().collect();
    println!("btree union {:?}", bs.union(&other).collect::<Vec<_>>());
    println!("btree inter {:?}", bs.intersection(&other).collect::<Vec<_>>());

    let mut dq: VecDeque<i32> = VecDeque::new();
    dq.push_back(1);
    dq.push_back(2);
    dq.push_front(0);
    dq.push_front(-1);
    println!("deque {:?} front {:?} back {:?}", dq, dq.front(), dq.back());
    println!("pop {:?} {:?}", dq.pop_front(), dq.pop_back());
    dq.extend([5, 6, 7]);
    dq.rotate_left(2);
    println!("rotated {:?} len {} [1]={}", dq, dq.len(), dq[1]);
    if let Some(x) = dq.get_mut(0) {
        *x = 100;
    }
    println!("contains 6 {} iter rev {:?}", dq.contains(&6), dq.iter().rev().collect::<Vec<_>>());
    let v: Vec<i32> = dq.into_iter().collect();
    println!("as vec {:?}", v);
    let mut window: VecDeque<i32> = VecDeque::with_capacity(3);
    let mut maxes = vec![];
    for x in [4, 8, 1, 9, 3, 3, 7] {
        if window.len() == 3 {
            window.pop_front();
        }
        window.push_back(x);
        maxes.push(*window.iter().max().unwrap());
    }
    println!("sliding max {:?}", maxes);

    let mut heap = BinaryHeap::new();
    for x in [5, 1, 8, 3, 9, 2] {
        heap.push(x);
    }
    println!("peek {:?} len {}", heap.peek(), heap.len());
    let mut out = vec![];
    while let Some(x) = heap.pop() {
        out.push(x);
    }
    println!("max-heap order {:?}", out);
    let mut minheap: BinaryHeap<Reverse<(u32, &str)>> = BinaryHeap::new();
    minheap.push(Reverse((3, "c")));
    minheap.push(Reverse((1, "a")));
    minheap.push(Reverse((2, "b")));
    while let Some(Reverse((p, name))) = minheap.pop() {
        print!("{}:{} ", p, name);
    }
    println!();
    let heap2: BinaryHeap<i32> = vec![3, 1, 4, 1, 5].into_iter().collect();
    println!("sorted vec {:?}", heap2.clone().into_sorted_vec());
    println!("into_vec len {}", heap2.into_vec().len());
    let mut kth = BinaryHeap::new();
    for x in [7, 2, 9, 4, 6, 1] {
        kth.push(Reverse(x));
        if kth.len() > 3 {
            kth.pop();
        }
    }
    println!("3rd largest {:?}", kth.peek().map(|r| r.0));
    println!("reverse cmp {} {:?}", Reverse(1) > Reverse(2), Reverse(5).0);
}
