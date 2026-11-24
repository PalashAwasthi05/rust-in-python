use std::collections::HashMap;
use std::fmt;
use std::ops::{Deref, DerefMut, Index};

#[derive(Debug, Clone, Default)]
struct Inventory {
    items: Vec<(String, u32)>,
}

impl Inventory {
    fn add(&mut self, name: &str, qty: u32) {
        match self.items.iter_mut().find(|(n, _)| n == name) {
            Some((_, q)) => *q += qty,
            None => self.items.push((name.to_string(), qty)),
        }
    }
}

impl<'a> IntoIterator for &'a Inventory {
    type Item = &'a (String, u32);
    type IntoIter = std::slice::Iter<'a, (String, u32)>;
    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

impl IntoIterator for Inventory {
    type Item = (String, u32);
    type IntoIter = std::vec::IntoIter<(String, u32)>;
    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl FromIterator<(String, u32)> for Inventory {
    fn from_iter<I: IntoIterator<Item = (String, u32)>>(iter: I) -> Self {
        let mut inv = Inventory::default();
        for (n, q) in iter {
            inv.add(&n, q);
        }
        inv
    }
}

impl Extend<(String, u32)> for Inventory {
    fn extend<I: IntoIterator<Item = (String, u32)>>(&mut self, iter: I) {
        for (n, q) in iter {
            self.add(&n, q);
        }
    }
}

impl Index<&str> for Inventory {
    type Output = u32;
    fn index(&self, name: &str) -> &u32 {
        self.items.iter().find(|(n, _)| n == name).map(|(_, q)| q).unwrap_or(&0)
    }
}

impl fmt::Display for Inventory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, (n, q)) in self.items.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}x{}", q, n)?;
        }
        Ok(())
    }
}

struct Sorted(Vec<i32>);

impl From<Vec<i32>> for Sorted {
    fn from(mut v: Vec<i32>) -> Self {
        v.sort_unstable();
        Sorted(v)
    }
}

impl Deref for Sorted {
    type Target = Vec<i32>;
    fn deref(&self) -> &Vec<i32> {
        &self.0
    }
}

struct Counter {
    inner: HashMap<char, usize>,
}

impl Deref for Counter {
    type Target = HashMap<char, usize>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Counter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

fn main() {
    let mut inv: Inventory = vec![("apple".to_string(), 3), ("pear".to_string(), 1), ("apple".to_string(), 2)]
        .into_iter()
        .collect();
    println!("{}", inv);
    inv.extend(vec![("fig".to_string(), 7), ("pear".to_string(), 4)]);
    println!("{}", inv);
    let mut total = 0;
    for (name, q) in &inv {
        if name.len() > 3 {
            total += q;
        }
    }
    println!("total long-named {}", total);
    println!("apple={} kiwi={}", inv["apple"], inv["kiwi"]);
    let names: Vec<String> = inv.clone().into_iter().map(|(n, _)| n).collect();
    println!("{:?}", names);
    let biggest = (&inv).into_iter().max_by_key(|(_, q)| *q).unwrap();
    println!("{:?}", biggest);
    let s: Sorted = vec![5, -1, 3, 3, 0].into();
    println!("{:?} len={} first={:?} contains3={}", *s, s.len(), s.first(), s.contains(&3));
    println!("{}", s.iter().map(|x| x.to_string()).collect::<Vec<_>>().join("<="));
    let mut c = Counter { inner: HashMap::new() };
    for ch in "hello world".chars().filter(|c| c.is_alphabetic()) {
        *c.entry(ch).or_insert(0) += 1;
    }
    let mut top: Vec<(&char, &usize)> = c.iter().collect();
    top.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    println!("{:?} distinct={}", &top[..3], c.len());
}
