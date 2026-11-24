// Fixed-capacity ring buffer with const generic capacity, plus a growable one over Vec<Option<T>>.
use std::fmt;
use std::ops::Index;

struct Ring<T: Copy + Default, const N: usize> {
    buf: [T; N],
    start: usize,
    len: usize,
}

impl<T: Copy + Default, const N: usize> Ring<T, N> {
    fn new() -> Self {
        Ring { buf: [T::default(); N], start: 0, len: 0 }
    }

    fn capacity(&self) -> usize {
        N
    }

    fn is_full(&self) -> bool {
        self.len == N
    }

    fn push(&mut self, v: T) -> Option<T> {
        if self.is_full() {
            let old = self.buf[self.start];
            self.buf[self.start] = v;
            self.start = (self.start + 1) % N;
            Some(old)
        } else {
            let idx = (self.start + self.len) % N;
            self.buf[idx] = v;
            self.len += 1;
            None
        }
    }

    fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        let v = self.buf[self.start];
        self.start = (self.start + 1) % N;
        self.len -= 1;
        Some(v)
    }

    fn iter(&self) -> impl Iterator<Item = T> + '_ {
        (0..self.len).map(move |i| self.buf[(self.start + i) % N])
    }
}

impl<T: Copy + Default, const N: usize> Index<usize> for Ring<T, N> {
    type Output = T;
    fn index(&self, i: usize) -> &T {
        assert!(i < self.len, "index {} out of range for ring of length {}", i, self.len);
        &self.buf[(self.start + i) % N]
    }
}

impl<T: Copy + Default + fmt::Debug, const N: usize> fmt::Debug for Ring<T, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

struct GrowRing<T> {
    slots: Vec<Option<T>>,
    head: usize,
    len: usize,
}

impl<T> GrowRing<T> {
    fn new() -> Self {
        GrowRing { slots: Vec::new(), head: 0, len: 0 }
    }

    fn grow(&mut self) {
        let new_cap = if self.slots.is_empty() { 2 } else { self.slots.len() * 2 };
        let mut fresh: Vec<Option<T>> = (0..new_cap).map(|_| None).collect();
        let cap = self.slots.len();
        for i in 0..self.len {
            fresh[i] = self.slots[(self.head + i) % cap].take();
        }
        self.slots = fresh;
        self.head = 0;
    }

    fn push_back(&mut self, v: T) {
        if self.len == self.slots.len() {
            self.grow();
        }
        let cap = self.slots.len();
        self.slots[(self.head + self.len) % cap] = Some(v);
        self.len += 1;
    }

    fn push_front(&mut self, v: T) {
        if self.len == self.slots.len() {
            self.grow();
        }
        let cap = self.slots.len();
        self.head = (self.head + cap - 1) % cap;
        self.slots[self.head] = Some(v);
        self.len += 1;
    }

    fn pop_front(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        let v = self.slots[self.head].take();
        self.head = (self.head + 1) % self.slots.len();
        self.len -= 1;
        v
    }

    fn pop_back(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        let idx = (self.head + self.len) % self.slots.len();
        self.slots[idx].take()
    }

    fn get(&self, i: usize) -> Option<&T> {
        if i >= self.len {
            return None;
        }
        self.slots[(self.head + i) % self.slots.len()].as_ref()
    }
}

fn moving_average(samples: &[f64]) -> Vec<f64> {
    let mut window: Ring<f64, 3> = Ring::new();
    let mut out = Vec::new();
    for &s in samples {
        window.push(s);
        let sum: f64 = window.iter().sum();
        out.push(sum / window.len as f64);
    }
    out
}

fn main() {
    let mut r: Ring<i32, 4> = Ring::new();
    println!("cap {} empty pop {:?}", r.capacity(), r.pop());
    for i in 1..=6 {
        let overwritten = r.push(i);
        println!("push {} -> overwritten {:?}, ring {:?}", i, overwritten, r);
    }
    println!("r[0]={} r[3]={}", r[0], r[3]);
    println!("pop {:?} {:?}", r.pop(), r.pop());
    r.push(7);
    println!("ring {:?} len {} full {}", r, r.len, r.is_full());
    let doubled: Vec<i32> = r.iter().map(|x| x * 2).collect();
    println!("doubled {:?}", doubled);

    let avgs = moving_average(&[1.0, 2.0, 3.0, 4.0, 10.0, 0.5]);
    let shown: Vec<String> = avgs.iter().map(|a| format!("{:.2}", a)).collect();
    println!("moving avg {}", shown.join(", "));

    let mut g: GrowRing<String> = GrowRing::new();
    for w in ["c", "d", "e"] {
        g.push_back(w.to_string());
    }
    g.push_front("b".into());
    g.push_front("a".into());
    println!("cap {} len {}", g.slots.len(), g.len);
    let all: Vec<&String> = (0..g.len).filter_map(|i| g.get(i)).collect();
    println!("contents {:?}", all);
    println!("pop_front {:?} pop_back {:?}", g.pop_front(), g.pop_back());
    for i in 0..5 {
        g.push_back(format!("x{}", i));
    }
    let mut drained = Vec::new();
    while let Some(s) = g.pop_front() {
        drained.push(s);
    }
    println!("drained {:?} cap {}", drained, g.slots.len());
    println!("get(0) on empty: {:?}", g.get(0));

    let mut r2: Ring<u8, 2> = Ring::new();
    r2.push(9);
    println!("r2[0] = {}", r2[0]);
    println!("r2[1] = {}", r2[1]);
}
