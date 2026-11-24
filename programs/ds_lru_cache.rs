// LRU cache: HashMap from key to slot index plus an intrusive doubly linked list stored in a Vec.
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

const NIL: usize = usize::MAX;

struct Entry<K, V> {
    key: K,
    value: V,
    prev: usize,
    next: usize,
}

struct LruCache<K, V> {
    capacity: usize,
    map: HashMap<K, usize>,
    slots: Vec<Entry<K, V>>,
    head: usize, // most recently used
    tail: usize, // least recently used
    hits: u32,
    misses: u32,
}

impl<K: Hash + Eq + Clone + Debug, V: Debug> LruCache<K, V> {
    fn with_capacity(capacity: usize) -> Self {
        assert!(capacity > 0, "capacity must be positive");
        LruCache {
            capacity,
            map: HashMap::with_capacity(capacity),
            slots: Vec::with_capacity(capacity),
            head: NIL,
            tail: NIL,
            hits: 0,
            misses: 0,
        }
    }

    fn detach(&mut self, i: usize) {
        let (prev, next) = (self.slots[i].prev, self.slots[i].next);
        if prev != NIL {
            self.slots[prev].next = next;
        } else {
            self.head = next;
        }
        if next != NIL {
            self.slots[next].prev = prev;
        } else {
            self.tail = prev;
        }
    }

    fn attach_front(&mut self, i: usize) {
        self.slots[i].prev = NIL;
        self.slots[i].next = self.head;
        if self.head != NIL {
            self.slots[self.head].prev = i;
        }
        self.head = i;
        if self.tail == NIL {
            self.tail = i;
        }
    }

    fn get(&mut self, key: &K) -> Option<&V> {
        match self.map.get(key).copied() {
            Some(i) => {
                self.hits += 1;
                self.detach(i);
                self.attach_front(i);
                Some(&self.slots[i].value)
            }
            None => {
                self.misses += 1;
                None
            }
        }
    }

    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let i = *self.map.get(key)?;
        self.detach(i);
        self.attach_front(i);
        Some(&mut self.slots[i].value)
    }

    fn put(&mut self, key: K, value: V) -> Option<(K, V)> {
        if let Some(&i) = self.map.get(&key) {
            self.slots[i].value = value;
            self.detach(i);
            self.attach_front(i);
            return None;
        }
        if self.slots.len() < self.capacity {
            let i = self.slots.len();
            self.slots.push(Entry { key: key.clone(), value, prev: NIL, next: NIL });
            self.map.insert(key, i);
            self.attach_front(i);
            None
        } else {
            let i = self.tail;
            self.detach(i);
            let old_key = std::mem::replace(&mut self.slots[i].key, key.clone());
            let old_value = std::mem::replace(&mut self.slots[i].value, value);
            self.map.remove(&old_key);
            self.map.insert(key, i);
            self.attach_front(i);
            Some((old_key, old_value))
        }
    }

    fn order(&self) -> Vec<&K> {
        let mut out = Vec::new();
        let mut i = self.head;
        while i != NIL {
            out.push(&self.slots[i].key);
            i = self.slots[i].next;
        }
        out
    }

    fn len(&self) -> usize {
        self.map.len()
    }
}

fn fib_memo(n: u64, cache: &mut LruCache<u64, u64>) -> u64 {
    if n < 2 {
        return n;
    }
    if let Some(&v) = cache.get(&n) {
        return v;
    }
    let v = fib_memo(n - 1, cache) + fib_memo(n - 2, cache);
    cache.put(n, v);
    v
}

fn main() {
    let mut c: LruCache<&str, i32> = LruCache::with_capacity(3);
    c.put("a", 1);
    c.put("b", 2);
    c.put("c", 3);
    println!("order {:?}", c.order());
    println!("get a -> {:?}", c.get(&"a"));
    println!("order {:?}", c.order());
    let evicted = c.put("d", 4);
    println!("put d evicted {:?}; order {:?}", evicted, c.order());
    println!("get b -> {:?}", c.get(&"b"));
    if let Some(v) = c.get_mut(&"c") {
        *v *= 10;
    }
    println!("after get_mut c: order {:?}", c.order());
    println!("c = {:?}", c.get(&"c"));
    println!("update a: {:?}", c.put("a", 100));
    println!("order {:?}", c.order());
    for (k, v) in [("e", 5), ("f", 6)] {
        println!("put {} evicted {:?}", k, c.put(k, v));
    }
    println!("final order {:?} len {} hits {} misses {}", c.order(), c.len(), c.hits, c.misses);

    let mut memo = LruCache::with_capacity(4);
    println!("fib(60) = {}", fib_memo(60, &mut memo));
    println!("memo order {:?} hits {} misses {}", memo.order(), memo.hits, memo.misses);

    let mut strings: LruCache<String, Vec<usize>> = LruCache::with_capacity(2);
    let text = "one two three two one four two";
    for (pos, w) in text.split(' ').enumerate() {
        match strings.get_mut(&w.to_string()) {
            Some(positions) => positions.push(pos),
            None => {
                if let Some((k, v)) = strings.put(w.to_string(), vec![pos]) {
                    println!("evicted {} with positions {:?}", k, v);
                }
            }
        }
    }
    for k in strings.order() {
        println!("{} -> {:?}", k, strings.map.get(k).map(|&i| &strings.slots[i].value));
    }
}
