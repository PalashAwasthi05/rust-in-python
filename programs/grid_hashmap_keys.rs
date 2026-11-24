// Arrays as HashMap/HashSet/BTreeMap keys, sorting arrays of arrays.
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

fn neighbors(p: [i32; 2]) -> [[i32; 2]; 4] {
    let [x, y] = p;
    [[x + 1, y], [x - 1, y], [x, y + 1], [x, y - 1]]
}

fn main() {
    let mut visits: HashMap<[i32; 2], u32> = HashMap::new();
    let path = [[0, 0], [1, 0], [1, 1], [0, 0], [1, 1], [0, 0]];
    for p in path.iter() {
        *visits.entry(*p).or_insert(0) += 1;
    }
    let mut counts: Vec<([i32; 2], u32)> = visits.into_iter().collect();
    counts.sort();
    println!("{:?}", counts);

    let mut seen: HashSet<[i32; 2]> = HashSet::new();
    let mut frontier = vec![[0, 0]];
    seen.insert([0, 0]);
    for _ in 0..2 {
        let mut next = Vec::new();
        for p in frontier {
            for n in neighbors(p) {
                if seen.insert(n) {
                    next.push(n);
                }
            }
        }
        frontier = next;
    }
    println!("reached {}", seen.len());
    println!("has [2,0]: {} has [2,1]: {}", seen.contains(&[2, 0]), seen.contains(&[2, 1]));

    let mut ordered: BTreeSet<[i32; 2]> = seen.into_iter().collect();
    ordered.retain(|p| p[0] >= 0 && p[1] >= 0);
    println!("{:?}", ordered);

    let mut anagrams: BTreeMap<[u8; 26], Vec<&str>> = BTreeMap::new();
    for w in ["listen", "silent", "enlist", "google", "gogole", "cat", "act", "tac", "dog"] {
        let mut key = [0u8; 26];
        for b in w.bytes() {
            key[(b - b'a') as usize] += 1;
        }
        anagrams.entry(key).or_default().push(w);
    }
    let mut groups: Vec<Vec<&str>> = anagrams.into_values().collect();
    groups.sort_by_key(|g| std::cmp::Reverse(g.len()));
    for g in &groups {
        println!("{:?}", g);
    }

    let mut tiles: HashMap<[char; 3], usize> = HashMap::new();
    tiles.insert(['a', 'b', 'c'], 1);
    tiles.insert(['x', 'y', 'z'], 2);
    println!("{:?} {:?}", tiles.get(&['a', 'b', 'c']), tiles.get(&['c', 'b', 'a']));

    let mut boards = vec![[3, 1, 2], [1, 2, 3], [1, 1, 9], [2, 0, 0]];
    boards.sort();
    println!("{:?}", boards);
    boards.sort_by(|a, b| b.iter().sum::<i32>().cmp(&a.iter().sum::<i32>()).then(a.cmp(b)));
    println!("{:?}", boards);
    boards.dedup_by_key(|b| b.iter().sum::<i32>());
    println!("{:?}", boards);
}
