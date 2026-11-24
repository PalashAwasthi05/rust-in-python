// Fixed-width bit set over Vec<u64> with set-algebra operator traits, an iterator over members, and a sieve.
use std::fmt;
use std::ops::{BitAnd, BitOr, BitXor, Not, Sub};

#[derive(Clone, PartialEq, Eq, Hash)]
struct BitSet {
    bits: usize,
    words: Vec<u64>,
}

impl BitSet {
    fn new(bits: usize) -> Self {
        BitSet { bits, words: vec![0; (bits + 63) / 64] }
    }

    fn full(bits: usize) -> Self {
        let mut s = BitSet::new(bits);
        for w in s.words.iter_mut() {
            *w = !0;
        }
        s.trim();
        s
    }

    fn trim(&mut self) {
        let extra = self.words.len() * 64 - self.bits;
        if extra > 0 {
            if let Some(last) = self.words.last_mut() {
                *last &= u64::MAX >> extra;
            }
        }
    }

    fn insert(&mut self, i: usize) -> bool {
        assert!(i < self.bits, "bit {} out of range", i);
        let (w, b) = (i / 64, i % 64);
        let was = self.words[w] >> b & 1 == 1;
        self.words[w] |= 1u64 << b;
        !was
    }

    fn remove(&mut self, i: usize) -> bool {
        let (w, b) = (i / 64, i % 64);
        let was = self.words[w] & (1 << b) != 0;
        self.words[w] &= !(1u64 << b);
        was
    }

    fn contains(&self, i: usize) -> bool {
        i < self.bits && self.words[i / 64] & (1 << (i % 64)) != 0
    }

    fn len(&self) -> u32 {
        self.words.iter().map(|w| w.count_ones()).sum()
    }

    fn iter(&self) -> Members<'_> {
        Members { set: self, word: 0, current: self.words.first().copied().unwrap_or(0) }
    }

    fn zip_with(&self, other: &BitSet, f: impl Fn(u64, u64) -> u64) -> BitSet {
        assert_eq!(self.bits, other.bits);
        let words = self.words.iter().zip(&other.words).map(|(&a, &b)| f(a, b)).collect();
        let mut out = BitSet { bits: self.bits, words };
        out.trim();
        out
    }

    fn is_subset(&self, other: &BitSet) -> bool {
        self.words.iter().zip(other.words.iter()).all(|(a, b)| a & !b == 0)
    }
}

struct Members<'a> {
    set: &'a BitSet,
    word: usize,
    current: u64,
}

impl<'a> Iterator for Members<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        loop {
            if self.current != 0 {
                let tz = self.current.trailing_zeros() as usize;
                self.current &= self.current - 1;
                return Some(self.word * 64 + tz);
            }
            self.word += 1;
            if self.word >= self.set.words.len() {
                return None;
            }
            self.current = self.set.words[self.word];
        }
    }
}

impl<'a> IntoIterator for &'a BitSet {
    type Item = usize;
    type IntoIter = Members<'a>;
    fn into_iter(self) -> Members<'a> {
        self.iter()
    }
}

impl BitAnd for &BitSet {
    type Output = BitSet;
    fn bitand(self, rhs: &BitSet) -> BitSet {
        self.zip_with(rhs, |a, b| a & b)
    }
}

impl BitOr for &BitSet {
    type Output = BitSet;
    fn bitor(self, rhs: &BitSet) -> BitSet {
        self.zip_with(rhs, |a, b| a | b)
    }
}

impl BitXor for &BitSet {
    type Output = BitSet;
    fn bitxor(self, rhs: &BitSet) -> BitSet {
        self.zip_with(rhs, |a, b| a ^ b)
    }
}

impl Sub for &BitSet {
    type Output = BitSet;
    fn sub(self, rhs: &BitSet) -> BitSet {
        self.zip_with(rhs, |a, b| a & !b)
    }
}

impl Not for &BitSet {
    type Output = BitSet;
    fn not(self) -> BitSet {
        let mut out = BitSet { bits: self.bits, words: self.words.iter().map(|w| !w).collect() };
        out.trim();
        out
    }
}

impl fmt::Debug for BitSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl fmt::Display for BitSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..self.bits {
            f.write_str(if self.contains(i) { "1" } else { "0" })?;
        }
        Ok(())
    }
}

fn sieve(n: usize) -> BitSet {
    let mut composite = BitSet::new(n + 1);
    composite.insert(0);
    composite.insert(1);
    let mut i = 2;
    while i * i <= n {
        if !composite.contains(i) {
            let mut j = i * i;
            while j <= n {
                composite.insert(j);
                j += i;
            }
        }
        i += 1;
    }
    !&composite
}

fn main() {
    let mut a = BitSet::new(12);
    for i in [1, 3, 5, 7, 9, 11] {
        a.insert(i);
    }
    let mut b = BitSet::new(12);
    for i in (0..12).filter(|i| i % 3 == 0) {
        b.insert(i);
    }
    println!("a = {} {:?} len {}", a, a, a.len());
    println!("b = {} {:?} len {}", b, b, b.len());
    println!("a & b = {:?}", &a & &b);
    println!("a | b = {:?}", &a | &b);
    println!("a ^ b = {:?}", &a ^ &b);
    println!("a - b = {:?}", &a - &b);
    println!("!a    = {} {:?}", !&a, !&a);
    println!("insert 3 again: {} remove 3: {} remove 3 again: {}", a.insert(3), a.remove(3), a.remove(3));
    println!("contains 5: {} contains 6: {} contains 99: {}", a.contains(5), a.contains(6), a.contains(99));
    let small = &a & &b;
    println!("(a&b) subset of a: {} a subset of b: {}", small.is_subset(&a), a.is_subset(&b));

    let full = BitSet::full(70);
    let hex: Vec<String> = full.words.iter().map(|w| format!("{:#x}", w)).collect();
    println!("full(70): len {} words {:?}", full.len(), hex);
    let mut total = 0;
    for i in &full {
        total += i;
    }
    println!("sum of members {}", total);

    let primes = sieve(200);
    let list: Vec<usize> = primes.iter().collect();
    println!("{} primes <= 200, last five {:?}", primes.len(), &list[list.len() - 5..]);
    let twins: Vec<(usize, usize)> = primes.iter().filter(|&p| primes.contains(p + 2)).map(|p| (p, p + 2)).take(6).collect();
    println!("twin primes {:?}", twins);
    let mut seen = std::collections::HashSet::new();
    seen.insert(a.clone());
    seen.insert(b.clone());
    seen.insert(a.clone());
    println!("distinct sets {}", seen.len());
    println!("{}", BitSet::new(0));
    a.insert(12);
}
