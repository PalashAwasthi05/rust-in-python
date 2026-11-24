use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

// Equality and hashing ignore the cached `label` field.
#[derive(Debug, Clone)]
struct Account {
    id: u64,
    email: String,
    label: String,
}

impl PartialEq for Account {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.email.to_lowercase() == other.email.to_lowercase()
    }
}

impl Eq for Account {}

impl Hash for Account {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.email.to_lowercase().hash(state);
    }
}

// Case-insensitive string key.
#[derive(Debug, Clone)]
struct NoCase(String);

impl PartialEq for NoCase {
    fn eq(&self, other: &NoCase) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}
impl Eq for NoCase {}
impl Hash for NoCase {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for c in self.0.chars() {
            state.write_u32(c.to_ascii_lowercase() as u32);
        }
    }
}

// Unordered pair: (a, b) == (b, a).
#[derive(Debug, Clone, Copy)]
struct Edge(u32, u32);

impl PartialEq for Edge {
    fn eq(&self, other: &Edge) -> bool {
        (self.0 == other.0 && self.1 == other.1) || (self.0 == other.1 && self.1 == other.0)
    }
}
impl Eq for Edge {}
impl Hash for Edge {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let (lo, hi) = if self.0 < self.1 { (self.0, self.1) } else { (self.1, self.0) };
        lo.hash(state);
        hi.hash(state);
    }
}

fn hash_of<T: Hash>(t: &T) -> u64 {
    let mut h = DefaultHasher::new();
    t.hash(&mut h);
    h.finish()
}

fn main() {
    let a = Account { id: 1, email: "Ann@Example.com".to_string(), label: "first".to_string() };
    let b = Account { id: 1, email: "ann@example.com".to_string(), label: "second".to_string() };
    let c = Account { id: 2, email: "ann@example.com".to_string(), label: "third".to_string() };
    println!("a == b: {}", a == b);
    println!("a == c: {}", a == c);
    println!("hash a == hash b: {}", hash_of(&a) == hash_of(&b));
    println!("hash a == hash c: {}", hash_of(&a) == hash_of(&c));

    let mut set = HashSet::new();
    println!("insert a: {}", set.insert(a.clone()));
    println!("insert b: {}", set.insert(b.clone()));
    println!("insert c: {}", set.insert(c.clone()));
    println!("set len: {}", set.len());
    let kept = set.get(&b).unwrap();
    println!("kept label: {}", kept.label);
    println!("contains b: {}", set.contains(&b));

    let mut scores: HashMap<Account, u32> = HashMap::new();
    scores.insert(a.clone(), 10);
    *scores.entry(b.clone()).or_insert(0) += 5;
    scores.insert(c.clone(), 1);
    let mut lines: Vec<String> = scores.iter().map(|(k, v)| format!("{}:{}={}", k.id, k.email, v)).collect();
    lines.sort();
    println!("{:?}", lines);

    let mut env: HashMap<NoCase, i32> = HashMap::new();
    env.insert(NoCase("PATH".to_string()), 1);
    env.insert(NoCase("Home".to_string()), 2);
    env.insert(NoCase("path".to_string()), 3);
    println!("env len {}", env.len());
    println!("PaTh -> {:?}", env.get(&NoCase("PaTh".to_string())));
    println!("HOME -> {:?}", env.get(&NoCase("HOME".to_string())));
    println!("nocase hash eq: {}", hash_of(&NoCase("AbC".to_string())) == hash_of(&NoCase("aBc".to_string())));

    let edges = [Edge(1, 2), Edge(2, 1), Edge(3, 4), Edge(4, 3), Edge(1, 3), Edge(2, 2)];
    let unique: HashSet<Edge> = edges.iter().cloned().collect();
    println!("unique edges: {}", unique.len());
    let mut weights: HashMap<Edge, u32> = HashMap::new();
    for (i, e) in edges.iter().enumerate() {
        *weights.entry(*e).or_insert(0) += i as u32;
    }
    let mut w: Vec<(u32, u32, u32)> = weights
        .iter()
        .map(|(e, w)| (e.0.min(e.1), e.0.max(e.1), *w))
        .collect();
    w.sort();
    println!("{:?}", w);
    println!("Edge(5,6) == Edge(6,5): {}", Edge(5, 6) == Edge(6, 5));
    println!("same tuple hash: {}", hash_of(&(1, "x")) == hash_of(&(1, "x")));
    println!("diff tuple hash: {}", hash_of(&(1, "x")) == hash_of(&(2, "x")));
}
