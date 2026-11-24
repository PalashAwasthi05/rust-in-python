// FromIterator, Extend, Sum and Product implemented for user types.
use std::collections::HashMap;
use std::fmt;
use std::iter::{FromIterator, Product, Sum};
use std::ops::{Add, Mul};

#[derive(Debug, Default)]
struct Bag {
    counts: HashMap<String, usize>,
    total: usize,
}

impl Bag {
    fn sorted(&self) -> Vec<(&String, &usize)> {
        let mut v: Vec<(&String, &usize)> = self.counts.iter().collect();
        v.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
        v
    }
}

impl<'a> FromIterator<&'a str> for Bag {
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        let mut bag = Bag::default();
        bag.extend(iter);
        bag
    }
}

impl<'a> Extend<&'a str> for Bag {
    fn extend<I: IntoIterator<Item = &'a str>>(&mut self, iter: I) {
        for word in iter {
            *self.counts.entry(word.to_string()).or_insert(0) += 1;
            self.total += 1;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Money {
    cents: i64,
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "${}.{:02}", self.cents / 100, self.cents % 100)
    }
}

impl Add for Money {
    type Output = Money;
    fn add(self, other: Money) -> Money {
        Money { cents: self.cents + other.cents }
    }
}

impl Sum for Money {
    fn sum<I: Iterator<Item = Money>>(iter: I) -> Money {
        iter.fold(Money { cents: 0 }, |a, b| a + b)
    }
}

impl<'a> Sum<&'a Money> for Money {
    fn sum<I: Iterator<Item = &'a Money>>(iter: I) -> Money {
        iter.fold(Money { cents: 0 }, |a, b| a + *b)
    }
}

#[derive(Debug, Clone, Copy)]
struct Scale(f64);

impl Mul for Scale {
    type Output = Scale;
    fn mul(self, rhs: Scale) -> Scale {
        Scale(self.0 * rhs.0)
    }
}

impl Product for Scale {
    fn product<I: Iterator<Item = Scale>>(iter: I) -> Scale {
        iter.fold(Scale(1.0), |a, b| a * b)
    }
}

#[derive(Debug)]
struct SortedVec(Vec<i32>);

impl FromIterator<i32> for SortedVec {
    fn from_iter<T: IntoIterator<Item = i32>>(iter: T) -> Self {
        let mut v: Vec<i32> = iter.into_iter().collect();
        v.sort();
        SortedVec(v)
    }
}

impl Extend<i32> for SortedVec {
    fn extend<T: IntoIterator<Item = i32>>(&mut self, iter: T) {
        for x in iter {
            let pos = self.0.partition_point(|&y| y < x);
            self.0.insert(pos, x);
        }
    }
}

fn main() {
    let text = "the cat and the dog and the bird";
    let bag: Bag = text.split(' ').collect();
    println!("total {} distinct {}", bag.total, bag.counts.len());
    for (word, n) in bag.sorted() {
        println!("{:>5} x{}", word, n);
    }
    let mut bag2 = text.split_whitespace().take(3).collect::<Bag>();
    bag2.extend(vec!["cat", "cow"]);
    println!("{:?}", bag2.sorted());

    let prices = vec![Money { cents: 199 }, Money { cents: 250 }, Money { cents: 1005 }];
    let total: Money = prices.iter().sum();
    println!("total {}", total);
    let owned_total: Money = prices.into_iter().filter(|m| m.cents > 200).sum();
    println!("over two dollars {}", owned_total);
    let none: Money = Vec::<Money>::new().into_iter().sum();
    println!("empty sum {}", none);

    let factor: Scale = vec![Scale(2.0), Scale(1.5), Scale(4.0)].into_iter().product();
    println!("factor {:?}", factor);

    let sv: SortedVec = vec![5, 3, 9, 1].into_iter().collect();
    println!("{:?}", sv);
    let mut sv = (1..4).rev().collect::<SortedVec>();
    sv.extend(vec![0, 2, 10]);
    println!("{:?}", sv);
    let from: SortedVec = SortedVec::from_iter(vec![3, 1, 2]);
    println!("{:?}", from);
}
