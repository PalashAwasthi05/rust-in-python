// An LRU cache with optional TTL built on an index-linked list in a Vec plus a HashMap,
// driven by a simulated clock and a workload of requests against a slow backend.
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

const NIL: usize = usize::MAX;

#[derive(Debug)]
struct Entry<K, V> {
    key: K,
    value: V,
    expires_at: Option<u64>,
    prev: usize,
    next: usize,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct Stats {
    hits: u32,
    misses: u32,
    expired: u32,
    evicted: u32,
}

impl Stats {
    fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 { 0.0 } else { self.hits as f64 / total as f64 * 100.0 }
    }
}

struct LruCache<K, V> {
    capacity: usize,
    ttl: Option<u64>,
    map: HashMap<K, usize>,
    slots: Vec<Option<Entry<K, V>>>,
    free: Vec<usize>,
    head: usize,
    tail: usize,
    stats: Stats,
}

impl<K: Hash + Eq + Clone + Debug, V: Clone + Debug> LruCache<K, V> {
    fn new(capacity: usize, ttl: Option<u64>) -> Self {
        assert!(capacity > 0);
        LruCache { capacity, ttl, map: HashMap::new(), slots: Vec::new(), free: vec![], head: NIL, tail: NIL, stats: Stats::default() }
    }

    fn entry(&self, idx: usize) -> &Entry<K, V> {
        self.slots[idx].as_ref().expect("live slot")
    }

    fn entry_mut(&mut self, idx: usize) -> &mut Entry<K, V> {
        self.slots[idx].as_mut().expect("live slot")
    }

    fn unlink(&mut self, idx: usize) {
        let (prev, next) = {
            let e = self.entry(idx);
            (e.prev, e.next)
        };
        if prev != NIL { self.entry_mut(prev).next = next; } else { self.head = next; }
        if next != NIL { self.entry_mut(next).prev = prev; } else { self.tail = prev; }
    }

    fn push_front(&mut self, idx: usize) {
        let old = self.head;
        {
            let e = self.entry_mut(idx);
            e.prev = NIL;
            e.next = old;
        }
        if old != NIL {
            self.entry_mut(old).prev = idx;
        } else {
            self.tail = idx;
        }
        self.head = idx;
    }

    fn remove_slot(&mut self, idx: usize) -> Entry<K, V> {
        self.unlink(idx);
        let e = self.slots[idx].take().unwrap();
        self.map.remove(&e.key);
        self.free.push(idx);
        e
    }

    fn get(&mut self, key: &K, now: u64) -> Option<V> {
        let idx = match self.map.get(key) {
            Some(&i) => i,
            None => {
                self.stats.misses += 1;
                return None;
            }
        };
        if matches!(self.entry(idx).expires_at, Some(t) if t <= now) {
            self.remove_slot(idx);
            self.stats.expired += 1;
            self.stats.misses += 1;
            return None;
        }
        self.unlink(idx);
        self.push_front(idx);
        self.stats.hits += 1;
        Some(self.entry(idx).value.clone())
    }

    fn put(&mut self, key: K, value: V, now: u64) -> Option<(K, V)> {
        let expires_at = self.ttl.map(|t| now + t);
        if let Some(&idx) = self.map.get(&key) {
            let e = self.entry_mut(idx);
            e.value = value;
            e.expires_at = expires_at;
            self.unlink(idx);
            self.push_front(idx);
            return None;
        }
        let mut evicted = None;
        if self.map.len() == self.capacity {
            let victim = self.tail;
            let e = self.remove_slot(victim);
            self.stats.evicted += 1;
            evicted = Some((e.key, e.value));
        }
        let entry = Entry { key: key.clone(), value, expires_at, prev: NIL, next: NIL };
        let idx = match self.free.pop() {
            Some(i) => {
                self.slots[i] = Some(entry);
                i
            }
            None => {
                self.slots.push(Some(entry));
                self.slots.len() - 1
            }
        };
        self.map.insert(key, idx);
        self.push_front(idx);
        evicted
    }

    fn purge_expired(&mut self, now: u64) -> Vec<K> {
        let stale: Vec<usize> = self
            .map
            .values()
            .copied()
            .filter(|&i| self.entry(i).expires_at.map_or(false, |t| t <= now))
            .collect();
        let mut keys: Vec<K> = stale.into_iter().map(|i| self.remove_slot(i).key).collect();
        self.stats.expired += keys.len() as u32;
        keys.sort_by_key(|k| format!("{:?}", k));
        keys
    }

    fn keys_mru(&self) -> Vec<K> {
        let mut out = Vec::with_capacity(self.map.len());
        let mut cur = self.head;
        while cur != NIL {
            let e = self.entry(cur);
            out.push(e.key.clone());
            cur = e.next;
        }
        out
    }
}

/// A pretend slow service: computing a value costs time on the simulated clock.
struct Backend {
    calls: u32,
}

impl Backend {
    fn fetch(&mut self, key: u32) -> (String, u64) {
        self.calls += 1;
        let cost = 10 + (key as u64 % 7) * 3;
        (format!("user-{:03}:{}", key, key.wrapping_mul(2654435761) % 1000), cost)
    }
}

fn simulate(label: &str, capacity: usize, ttl: Option<u64>, requests: &[u32]) -> (Stats, u64) {
    let mut cache: LruCache<u32, String> = LruCache::new(capacity, ttl);
    let mut backend = Backend { calls: 0 };
    let mut clock = 0u64;
    for (n, &key) in requests.iter().enumerate() {
        clock += 1;
        let value = match cache.get(&key, clock) {
            Some(v) => v,
            None => {
                let (v, cost) = backend.fetch(key);
                clock += cost;
                if let Some((old, _)) = cache.put(key, v.clone(), clock) {
                    if n < 12 {
                        println!("  [{}] evicted {} to make room for {}", label, old, key);
                    }
                }
                v
            }
        };
        if n % 9 == 0 {
            println!("  [{}] t={:>4} req {:>2} -> {}", label, clock, key, value);
        }
    }
    let purged = cache.purge_expired(clock + 1000);
    println!("  [{}] final MRU order {:?}, purged {} at end", label, cache.keys_mru(), purged.len());
    assert_eq!(backend.calls, cache.stats.misses);
    (cache.stats, clock)
}

fn main() {
    // Deterministic pseudo-random workload with a hot set.
    let mut seed: u32 = 12345;
    let mut next = || {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        seed
    };
    let requests: Vec<u32> = (0..60)
        .map(|_| {
            let r = next();
            if r % 10 < 7 { r % 5 } else { 5 + r % 20 }
        })
        .collect();
    println!("workload head: {:?}", &requests[..15]);

    let configs: [(&str, usize, Option<u64>); 4] = [
        ("tiny", 2, None),
        ("lru-5", 5, None),
        ("lru-5-ttl", 5, Some(60)),
        ("big", 32, None),
    ];
    let mut results = Vec::new();
    for &(label, cap, ttl) in configs.iter() {
        println!("{} (capacity {}, ttl {:?})", label, cap, ttl);
        let (stats, clock) = simulate(label, cap, ttl, &requests);
        results.push((label, stats, clock));
    }
    println!("{:<10} {:>5} {:>6} {:>7} {:>7} {:>7} {:>6}", "config", "hits", "misses", "expired", "evicted", "rate", "time");
    for (label, s, clock) in &results {
        println!("{:<10} {:>5} {:>6} {:>7} {:>7} {:>6.1}% {:>6}", label, s.hits, s.misses, s.expired, s.evicted, s.hit_rate(), clock);
    }
    let best = results.iter().min_by_key(|r| r.2).unwrap();
    println!("fastest: {} at t={}", best.0, best.2);

    let mut c: LruCache<&str, i32> = LruCache::new(3, Some(5));
    c.put("a", 1, 0);
    c.put("b", 2, 1);
    c.put("c", 3, 2);
    println!("{:?} {:?}", c.get(&"a", 3), c.keys_mru());
    println!("{:?}", c.put("d", 4, 4));
    println!("{:?} {:?} {:?}", c.get(&"a", 6), c.get(&"d", 6), c.keys_mru());
    println!("{:?} {:?}", c.purge_expired(9), c.stats);
}
