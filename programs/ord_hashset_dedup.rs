use std::collections::{BTreeSet, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Tag {
    name: String,
    weight: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Dir {
    N,
    E,
    S,
    W,
}

fn tag(name: &str, weight: u8) -> Tag {
    Tag { name: name.to_string(), weight }
}

fn main() {
    let tags = vec![tag("rust", 3), tag("python", 2), tag("rust", 3), tag("rust", 1), tag("go", 2), tag("python", 2)];
    let unique: HashSet<Tag> = tags.iter().cloned().collect();
    println!("unique count: {}", unique.len());
    let mut sorted: Vec<&Tag> = unique.iter().collect();
    sorted.sort();
    for t in &sorted {
        println!("{} {}", t.name, t.weight);
    }

    // Order-preserving dedup with a seen-set.
    let mut seen = HashSet::new();
    let mut ordered = Vec::new();
    for t in &tags {
        if seen.insert(t.clone()) {
            ordered.push(t.clone());
        }
    }
    let names: Vec<String> = ordered.iter().map(|t| format!("{}{}", t.name, t.weight)).collect();
    println!("{:?}", names);

    // retain-based dedup
    let mut seen2: HashSet<String> = HashSet::new();
    let mut by_name = tags.clone();
    by_name.retain(|t| seen2.insert(t.name.clone()));
    println!("{:?}", by_name.iter().map(|t| &t.name).collect::<Vec<_>>());

    // dedup after sort
    let mut v = tags.clone();
    v.sort();
    v.dedup();
    println!("{:?}", v);
    let mut nums = vec![5, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5];
    nums.sort_unstable();
    nums.dedup();
    println!("{:?}", nums);
    let mut by_key = tags.clone();
    by_key.sort_by(|a, b| a.name.cmp(&b.name));
    by_key.dedup_by_key(|t| t.name.clone());
    println!("{:?}", by_key);
    let mut by_pred = vec![1, 2, 4, 7, 8, 9, 15, 16];
    by_pred.dedup_by(|a, b| *a - *b == 1);
    println!("{:?}", by_pred);

    let path = [Dir::N, Dir::E, Dir::N, Dir::S, Dir::E, Dir::W, Dir::N];
    let dirs: HashSet<Dir> = path.iter().copied().collect();
    let mut present: Vec<&str> = Vec::new();
    for (d, s) in [(Dir::N, "N"), (Dir::E, "E"), (Dir::S, "S"), (Dir::W, "W")].iter() {
        if dirs.contains(d) {
            present.push(s);
        }
    }
    println!("{:?} {}", present, dirs.len());

    let a: HashSet<(i32, i32)> = [(0, 0), (1, 1), (2, 2)].iter().cloned().collect();
    let b: HashSet<(i32, i32)> = [(1, 1), (2, 2), (3, 3)].iter().cloned().collect();
    let mut both: Vec<_> = a.intersection(&b).collect();
    both.sort();
    println!("both {:?}", both);
    let mut either: Vec<_> = a.union(&b).cloned().collect();
    either.sort();
    println!("either {:?}", either);
    let mut sym: Vec<_> = a.symmetric_difference(&b).collect();
    sym.sort();
    println!("sym {:?}", sym);
    println!("subset: {}", [(1, 1)].iter().cloned().collect::<HashSet<_>>().is_subset(&a));
    println!("set eq: {}", a == [(2, 2), (0, 0), (1, 1)].iter().cloned().collect::<HashSet<_>>());

    let words: BTreeSet<&str> = "the quick brown fox jumps over the lazy dog the end".split(' ').collect();
    println!("{:?}", words);
    let strings: HashSet<String> = vec!["a".to_string(), "b".to_string(), "a".to_string()].into_iter().collect();
    println!("has a: {}, len {}", strings.contains("a"), strings.len());
}
