// Open-addressing hash map with linear probing and tombstones, using a hand-written FNV-1a Hasher.
use std::borrow::Borrow;
use std::fmt::Debug;
use std::hash::{BuildHasher, Hash, Hasher};

#[derive(Default, Clone, Copy)]
struct Fnv(u64);

impl Hasher for Fnv {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 ^= b as u64;
            self.0 = self.0.wrapping_mul(0x100000001b3);
        }
    }
}

#[derive(Default, Clone, Copy)]
struct FnvBuild;

impl BuildHasher for FnvBuild {
    type Hasher = Fnv;
    fn build_hasher(&self) -> Fnv {
        Fnv(0xcbf29ce484222325)
    }
}

#[derive(Debug, Clone)]
enum Slot<K, V> {
    Empty,
    Deleted,
    Full(K, V),
}

struct Table<K, V, S> {
    slots: Vec<Slot<K, V>>,
    len: usize,
    deleted: usize,
    build: S,
    probes: usize,
}

impl<K: Hash + Eq + Debug, V: Debug> Table<K, V, FnvBuild> {
    fn new() -> Self {
        Table { slots: (0..8).map(|_| Slot::Empty).collect(), len: 0, deleted: 0, build: FnvBuild, probes: 0 }
    }
}

impl<K: Hash + Eq + Debug, V: Debug, S: BuildHasher> Table<K, V, S> {
    fn hash_of<Q: Hash + ?Sized>(&self, key: &Q) -> usize {
        let mut h = self.build.build_hasher();
        key.hash(&mut h);
        h.finish() as usize
    }

    fn find_slot<Q>(&mut self, key: &Q) -> (usize, bool)
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let mask = self.slots.len() - 1;
        let mut i = self.hash_of(key) & mask;
        let mut first_free: Option<usize> = None;
        loop {
            self.probes += 1;
            match &self.slots[i] {
                Slot::Empty => return (first_free.unwrap_or(i), false),
                Slot::Deleted => {
                    if first_free.is_none() {
                        first_free = Some(i);
                    }
                }
                Slot::Full(k, _) if k.borrow() == key => return (i, true),
                Slot::Full(..) => {}
            }
            i = (i + 1) & mask;
        }
    }

    fn grow(&mut self) {
        let new_cap = self.slots.len() * 2;
        let old = std::mem::replace(&mut self.slots, (0..new_cap).map(|_| Slot::Empty).collect());
        self.len = 0;
        self.deleted = 0;
        for slot in old {
            if let Slot::Full(k, v) = slot {
                self.insert(k, v);
            }
        }
    }

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        if (self.len + self.deleted + 1) * 4 > self.slots.len() * 3 {
            self.grow();
        }
        let (i, found) = self.find_slot(&key);
        if found {
            if let Slot::Full(_, v) = &mut self.slots[i] {
                return Some(std::mem::replace(v, value));
            }
            unreachable!();
        }
        if let Slot::Deleted = self.slots[i] {
            self.deleted -= 1;
        }
        self.slots[i] = Slot::Full(key, value);
        self.len += 1;
        None
    }

    fn get<Q>(&mut self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        match self.find_slot(key) {
            (i, true) => match &self.slots[i] {
                Slot::Full(_, v) => Some(v),
                _ => None,
            },
            _ => None,
        }
    }

    fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let (i, found) = self.find_slot(key);
        if !found {
            return None;
        }
        self.len -= 1;
        self.deleted += 1;
        match std::mem::replace(&mut self.slots[i], Slot::Deleted) {
            Slot::Full(_, v) => Some(v),
            _ => unreachable!(),
        }
    }

    fn entries(&self) -> impl Iterator<Item = (&K, &V)> {
        self.slots.iter().filter_map(|s| match s {
            Slot::Full(k, v) => Some((k, v)),
            _ => None,
        })
    }

    fn layout(&self) -> String {
        self.slots
            .iter()
            .map(|s| match s {
                Slot::Empty => '.',
                Slot::Deleted => 'x',
                Slot::Full(..) => '#',
            })
            .collect()
    }
}

fn fnv(s: &str) -> u64 {
    let mut h = FnvBuild.build_hasher();
    h.write(s.as_bytes());
    h.finish()
}

fn main() {
    println!("fnv(\"\") = {:016x}", fnv(""));
    println!("fnv(\"a\") = {:016x}", fnv("a"));
    println!("fnv(\"foobar\") = {:016x}", fnv("foobar"));

    let mut t: Table<String, u32, FnvBuild> = Table::new();
    let words = "the cat sat on the mat and the dog sat on the log";
    for w in words.split(' ') {
        let n = t.get(w).copied().unwrap_or(0);
        t.insert(w.to_string(), n + 1);
    }
    println!("len {} cap {} layout {}", t.len, t.slots.len(), t.layout());
    let mut counts: Vec<(&String, &u32)> = t.entries().collect();
    counts.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    println!("counts {:?}", counts);
    println!("get the={:?}", t.get("the"));
    println!("get cow={:?}", t.get("cow"));
    println!("remove sat={:?} remove sat={:?}", t.remove("sat"), t.remove("sat"));
    println!("remove on={:?}; layout {}", t.remove("on"), t.layout());
    println!("insert sat -> {:?}; len {} deleted {}", t.insert("sat".into(), 9), t.len, t.deleted);
    println!("overwrite the -> {:?}", t.insert("the".to_string(), 0));

    let mut nums: Table<u64, u64, FnvBuild> = Table::new();
    for i in 0..200u64 {
        nums.insert(i * 7919 % 1009, i);
    }
    for i in 0..100u64 {
        nums.remove(&(i * 7919 % 1009));
    }
    let probes_before = nums.probes;
    let hits = (0..1009u64).filter(|k| nums.get(k).is_some()).count();
    println!("nums len {} cap {} hits {} avg probes/lookup {:.2}", nums.len, nums.slots.len(), hits, (nums.probes - probes_before) as f64 / 1009.0);
    let mut sum = 0;
    for (_, v) in nums.entries() {
        sum += v;
    }
    println!("sum of remaining values {}", sum);

    let mut tuple_keys: Table<(i32, char), &str, FnvBuild> = Table::new();
    tuple_keys.insert((1, 'a'), "one-a");
    tuple_keys.insert((2, 'b'), "two-b");
    println!("{:?}", tuple_keys.get(&(1, 'a')));
    println!("{:?}", tuple_keys.get(&(1, 'b')));
}
