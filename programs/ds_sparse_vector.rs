// Sparse vector backed by BTreeMap<usize, f64>: dot products, elementwise ops, iteration in index order.
use std::collections::BTreeMap;
use std::fmt;
use std::ops::{Add, Mul};

#[derive(Debug, Clone, PartialEq, Default)]
struct SparseVec {
    dim: usize,
    entries: BTreeMap<usize, f64>,
}

impl SparseVec {
    fn new(dim: usize) -> Self {
        SparseVec { dim, entries: BTreeMap::new() }
    }

    fn from_dense(values: &[f64]) -> Self {
        let mut v = SparseVec::new(values.len());
        for (i, &x) in values.iter().enumerate() {
            v.set(i, x);
        }
        v
    }

    fn set(&mut self, i: usize, x: f64) {
        assert!(i < self.dim, "index {} out of bounds for dim {}", i, self.dim);
        if x == 0.0 {
            self.entries.remove(&i);
        } else {
            self.entries.insert(i, x);
        }
    }

    fn get(&self, i: usize) -> f64 {
        self.entries.get(&i).copied().unwrap_or(0.0)
    }

    fn nnz(&self) -> usize {
        self.entries.len()
    }

    fn dot(&self, other: &SparseVec) -> f64 {
        let (small, large) = if self.nnz() <= other.nnz() { (self, other) } else { (other, self) };
        small.entries.iter().filter_map(|(i, a)| large.entries.get(i).map(|b| a * b)).sum()
    }

    fn norm(&self) -> f64 {
        self.dot(self).sqrt()
    }

    fn scale(&mut self, k: f64) {
        if k == 0.0 {
            self.entries.clear();
            return;
        }
        for v in self.entries.values_mut() {
            *v *= k;
        }
    }

    fn to_dense(&self) -> Vec<f64> {
        let mut out = vec![0.0; self.dim];
        for (&i, &x) in &self.entries {
            out[i] = x;
        }
        out
    }

    fn range_sum(&self, lo: usize, hi: usize) -> f64 {
        self.entries.range(lo..hi).map(|(_, v)| v).sum()
    }

    fn top_k(&self, k: usize) -> Vec<(usize, f64)> {
        let mut items: Vec<(usize, f64)> = self.entries.iter().map(|(&i, &v)| (i, v)).collect();
        items.sort_by(|a, b| b.1.abs().partial_cmp(&a.1.abs()).unwrap().then(a.0.cmp(&b.0)));
        items.truncate(k);
        items
    }
}

impl<'a> Add for &'a SparseVec {
    type Output = SparseVec;
    fn add(self, other: &SparseVec) -> SparseVec {
        assert_eq!(self.dim, other.dim);
        let mut out = self.clone();
        for (&i, &x) in other.entries.iter() {
            let e = out.entries.entry(i).or_insert(0.0);
            *e += x;
            if *e == 0.0 {
                out.entries.remove(&i);
            }
        }
        out
    }
}

impl Mul<f64> for SparseVec {
    type Output = SparseVec;
    fn mul(mut self, k: f64) -> SparseVec {
        self.scale(k);
        self
    }
}

impl fmt::Display for SparseVec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<dim {}:", self.dim)?;
        for (i, v) in &self.entries {
            write!(f, " {}:{}", i, v)?;
        }
        write!(f, ">")
    }
}

impl FromIterator<(usize, f64)> for SparseVec {
    fn from_iter<I: IntoIterator<Item = (usize, f64)>>(iter: I) -> Self {
        let mut entries = BTreeMap::new();
        let mut dim = 0;
        for (i, x) in iter {
            dim = dim.max(i + 1);
            if x != 0.0 {
                *entries.entry(i).or_insert(0.0) += x;
            }
        }
        SparseVec { dim, entries }
    }
}

fn main() {
    let mut a = SparseVec::new(10);
    a.set(1, 2.0);
    a.set(4, -1.5);
    a.set(9, 3.0);
    a.set(4, 0.0);
    a.set(7, 0.25);
    let b = SparseVec::from_dense(&[0.0, 1.0, 0.0, 0.0, 5.0, 0.0, 0.0, 4.0, 0.0, -2.0]);
    println!("a = {}", a);
    println!("b = {}", b);
    println!("nnz a={} b={}", a.nnz(), b.nnz());
    println!("a[1]={} a[4]={} a[9]={}", a.get(1), a.get(4), a.get(9));
    println!("a.b = {}", a.dot(&b));
    println!("|b| = {:.4}", b.norm());
    let c = &a + &b;
    println!("a + b = {}", c);
    println!("dense(a+b) = {:?}", c.to_dense());
    println!("sum over [2, 8) = {}", c.range_sum(2, 8));
    println!("top 2 = {:?}", c.top_k(2));

    let neg_b = b.clone() * -1.0;
    let zero = &b + &neg_b;
    println!("b - b = {} (nnz {}) equals empty? {}", zero, zero.nnz(), zero == SparseVec::new(10));

    let doc: SparseVec = vec![(3, 1.0), (0, 2.0), (3, 1.0), (12, 0.0), (5, -1.0)].into_iter().collect();
    println!("doc = {} dim {}", doc, doc.dim);
    let firsts: Vec<usize> = doc.entries.keys().cloned().collect();
    println!("keys {:?} first {:?} last {:?}", firsts, doc.entries.first_key_value(), doc.entries.last_key_value());

    let words = ["apple", "banana", "cherry", "date", "elder"];
    let vocab: BTreeMap<&str, usize> = words.iter().enumerate().map(|(i, w)| (*w, i)).collect();
    let embed = |text: &str| -> SparseVec {
        text.split(' ').filter_map(|w| vocab.get(w).map(|&i| (i, 1.0))).chain(std::iter::once((words.len() - 1, 0.0))).collect()
    };
    let x = embed("apple banana apple kiwi");
    let y = embed("banana cherry apple");
    let cos = x.dot(&y) / (x.norm() * y.norm());
    println!("x = {} y = {} cos = {:.4}", x, y, cos);

    let mut big = SparseVec::new(1_000_000);
    for i in (0..1_000_000).step_by(99_991) {
        big.set(i, (i % 7) as f64);
    }
    println!("big nnz {} sum {}", big.nnz(), big.range_sum(0, 1_000_000));
    big.scale(0.0);
    println!("after scale(0): {}", big.nnz());
}
