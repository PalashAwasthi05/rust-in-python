use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::hash::{Hash, Hasher};

// A key with a hand-written Hash/Eq that ignores ASCII case.
#[derive(Debug, Clone)]
struct Name(String);

impl PartialEq for Name {
    fn eq(&self, other: &Name) -> bool {
        self.0.to_lowercase() == other.0.to_lowercase()
    }
}
impl Eq for Name {}
impl Hash for Name {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_lowercase().hash(state);
    }
}

// Derived impls must use the field's hand-written impls.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Qualified {
    module: Name,
    depth: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Symbol {
    Local(Name),
    Global(Qualified),
    Anon,
}

fn digest<T: Hash>(t: &T) -> u64 {
    let mut h = DefaultHasher::new();
    t.hash(&mut h);
    h.finish()
}

fn name(s: &str) -> Name {
    Name(s.to_string())
}

fn main() {
    let a = Qualified { module: name("Core"), depth: 1 };
    let b = Qualified { module: name("CORE"), depth: 1 };
    let c = Qualified { module: name("core"), depth: 2 };
    println!("a == b: {}", a == b);
    println!("a == c: {}", a == c);
    println!("digest a == b: {}", digest(&a) == digest(&b));

    let mut seen: HashSet<Qualified> = HashSet::new();
    for q in [a.clone(), b.clone(), c.clone()].iter() {
        println!("insert {:?}: {}", q.module.0, seen.insert(q.clone()));
    }
    println!("seen: {}", seen.len());

    let mut table: HashMap<Symbol, u32> = HashMap::new();
    let symbols = vec![
        Symbol::Local(name("x")),
        Symbol::Local(name("X")),
        Symbol::Global(a.clone()),
        Symbol::Anon,
        Symbol::Global(b.clone()),
        Symbol::Global(c.clone()),
        Symbol::Anon,
        Symbol::Local(name("y")),
    ];
    for s in &symbols {
        *table.entry(s.clone()).or_insert(0) += 1;
    }
    println!("distinct symbols: {}", table.len());
    println!("x: {:?}", table.get(&Symbol::Local(name("x"))));
    println!("anon: {:?}", table.get(&Symbol::Anon));
    println!("core/1: {:?}", table.get(&Symbol::Global(Qualified { module: name("cOrE"), depth: 1 })));
    let mut report: Vec<String> = table
        .iter()
        .map(|(k, v)| match k {
            Symbol::Local(n) => format!("local {} x{}", n.0.to_lowercase(), v),
            Symbol::Global(q) => format!("global {}/{} x{}", q.module.0.to_lowercase(), q.depth, v),
            Symbol::Anon => format!("anon x{}", v),
        })
        .collect();
    report.sort();
    for line in &report {
        println!("{}", line);
    }

    let pairs: HashSet<(Name, i32)> = vec![(name("a"), 1), (name("A"), 1), (name("a"), 2)].into_iter().collect();
    println!("pairs: {}", pairs.len());
    let opt: HashSet<Option<Name>> = vec![Some(name("q")), None, Some(name("Q")), None].into_iter().collect();
    println!("opts: {}", opt.len());
    let lists: HashSet<Vec<Name>> = vec![vec![name("a"), name("b")], vec![name("A"), name("B")], vec![name("b")]]
        .into_iter()
        .collect();
    println!("lists: {}", lists.len());

    let mut grid: HashMap<(i32, i32), Vec<char>> = HashMap::new();
    for (i, ch) in "abcabc".chars().enumerate() {
        grid.entry(((i % 3) as i32, (i / 3) as i32 % 1)).or_default().push(ch);
    }
    let ordered: BTreeSet<_> = grid.iter().map(|(k, v)| (*k, v.iter().collect::<String>())).collect();
    println!("{:?}", ordered);
    let removed = grid.remove(&(1, 0));
    println!("removed {:?}, left {}", removed, grid.len());
}
