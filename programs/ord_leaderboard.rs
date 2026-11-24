use std::cmp::{Ordering, Reverse};
use std::collections::{BTreeMap, BinaryHeap, HashMap, HashSet};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Division {
    Gold,
    Silver,
    Bronze,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PlayerId(String);

#[derive(Debug, Clone)]
struct Entry {
    id: PlayerId,
    division: Division,
    points: u32,
    time_ms: u64,
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:<6} {:?} {:>4}pts {:>6}ms", self.id.0, self.division, self.points, self.time_ms)
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl Eq for Entry {}
impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
// Best first: division, then most points, then fastest time.
impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.division
            .cmp(&other.division)
            .then_with(|| other.points.cmp(&self.points))
            .then_with(|| self.time_ms.cmp(&other.time_ms))
    }
}

fn entry(id: &str, division: Division, points: u32, time_ms: u64) -> Entry {
    Entry { id: PlayerId(id.to_string()), division, points, time_ms }
}

fn main() {
    let entries = vec![
        entry("ana", Division::Silver, 120, 5300),
        entry("ben", Division::Gold, 90, 4100),
        entry("cleo", Division::Gold, 150, 6000),
        entry("dev", Division::Bronze, 200, 3000),
        entry("eli", Division::Gold, 150, 5900),
        entry("fay", Division::Silver, 120, 5300),
        entry("gus", Division::Bronze, 10, 9000),
    ];

    let mut ranked = entries.clone();
    ranked.sort();
    for (i, e) in ranked.iter().enumerate() {
        println!("{}. {}", i + 1, e);
    }
    println!("ana == fay (same rank): {}", entries[0] == entries[5]);
    println!("ana id == fay id: {}", entries[0].id == entries[5].id);

    let mut per_division: BTreeMap<Division, Vec<&Entry>> = BTreeMap::new();
    for e in &entries {
        per_division.entry(e.division).or_default().push(e);
    }
    for (division, list) in &per_division {
        let best = list.iter().min().unwrap();
        let total: u32 = list.iter().map(|e| e.points).sum();
        println!("{:?}: {} players, best {}, total {}", division, list.len(), best.id.0, total);
    }

    let mut heap: BinaryHeap<Reverse<Entry>> = entries.iter().cloned().map(Reverse).collect();
    let mut podium = Vec::new();
    while let Some(Reverse(e)) = heap.pop() {
        podium.push(e.id.0.clone());
        if podium.len() == 3 {
            break;
        }
    }
    println!("podium: {:?}", podium);

    let mut by_id: HashMap<PlayerId, &Entry> = HashMap::new();
    for e in &entries {
        by_id.insert(e.id.clone(), e);
    }
    let lookup = PlayerId("eli".to_string());
    println!("eli: {}", by_id[&lookup]);
    println!("zed: {:?}", by_id.get(&PlayerId("zed".to_string())).map(|e| e.points));

    let distinct_scores: HashSet<(Division, u32)> = entries.iter().map(|e| (e.division, e.points)).collect();
    let mut ds: Vec<_> = distinct_scores.into_iter().collect();
    ds.sort_by(|a, b| a.0.cmp(&b.0).then(b.1.cmp(&a.1)));
    println!("{:?}", ds);

    let fastest = entries.iter().min_by_key(|e| e.time_ms).unwrap();
    let slowest = entries.iter().max_by_key(|e| e.time_ms).unwrap();
    println!("fastest {} slowest {}", fastest.id.0, slowest.id.0);
    let top_points = entries.iter().max_by_key(|e| e.points).unwrap();
    println!("top points {}", top_points.id.0);
    let ties = entries.iter().filter(|e| e.points == 120).count();
    println!("ties at 120: {}", ties);
    let pos = ranked.binary_search_by(|e| e.cmp(&entry("?", Division::Gold, 100, 0)));
    println!("insert position for gold/100: {:?}", pos);
    let mut ranked_ids: Vec<&str> = ranked.iter().map(|e| e.id.0.as_str()).collect();
    ranked_ids.dedup_by_key(|s| s.len());
    println!("{:?}", ranked_ids);
}
