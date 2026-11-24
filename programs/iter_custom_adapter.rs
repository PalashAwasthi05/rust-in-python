// User-defined adapters wrapping any iterator, exposed through an extension trait.
use std::fmt::Debug;

struct Pairwise<I: Iterator> {
    inner: I,
    prev: Option<I::Item>,
}

impl<I> Iterator for Pairwise<I>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = (I::Item, I::Item);
    fn next(&mut self) -> Option<Self::Item> {
        if self.prev.is_none() {
            self.prev = self.inner.next();
        }
        let next = self.inner.next()?;
        let prev = self.prev.replace(next.clone())?;
        Some((prev, next))
    }
}

struct EveryOther<I> {
    inner: I,
}

impl<I: Iterator> Iterator for EveryOther<I> {
    type Item = I::Item;
    fn next(&mut self) -> Option<I::Item> {
        let item = self.inner.next();
        self.inner.next();
        item
    }
}

struct Batches<I> {
    inner: I,
    size: usize,
}

impl<I: Iterator> Iterator for Batches<I> {
    type Item = Vec<I::Item>;
    fn next(&mut self) -> Option<Vec<I::Item>> {
        let batch: Vec<I::Item> = self.inner.by_ref().take(self.size).collect();
        if batch.is_empty() { None } else { Some(batch) }
    }
}

struct Dedup<I: Iterator> {
    inner: std::iter::Peekable<I>,
}

impl<I> Iterator for Dedup<I>
where
    I: Iterator,
    I::Item: PartialEq,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<I::Item> {
        let item = self.inner.next()?;
        while self.inner.peek() == Some(&item) {
            self.inner.next();
        }
        Some(item)
    }
}

trait IterExt: Iterator + Sized {
    fn pairwise(self) -> Pairwise<Self> {
        Pairwise { inner: self, prev: None }
    }
    fn every_other(self) -> EveryOther<Self> {
        EveryOther { inner: self }
    }
    fn batches(self, size: usize) -> Batches<Self> {
        Batches { inner: self, size }
    }
    fn dedup_adjacent(self) -> Dedup<Self> {
        Dedup { inner: self.peekable() }
    }
    fn debug_all(self) -> String
    where
        Self::Item: Debug,
    {
        let parts: Vec<String> = self.map(|x| format!("{:?}", x)).collect();
        parts.join(" ")
    }
}

impl<I: Iterator> IterExt for I {}

struct Squares {
    n: u32,
}

impl Iterator for Squares {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        self.n += 1;
        Some(self.n * self.n)
    }
}

fn main() {
    let diffs: Vec<i32> = vec![1, 4, 9, 16, 25].into_iter().pairwise().map(|(a, b)| b - a).collect();
    println!("diffs {:?}", diffs);
    let words: Vec<(&str, &str)> = "a b c d".split(' ').pairwise().collect();
    println!("{:?}", words);
    println!("{}", (1..=9).every_other().debug_all());
    println!("{}", Squares { n: 0 }.every_other().take(4).debug_all());
    for batch in (1..=7).batches(3) {
        println!("batch {:?}", batch);
    }
    let sums: Vec<u32> = Squares { n: 0 }.batches(2).take(3).map(|b| b.iter().sum()).collect();
    println!("batch sums {:?}", sums);
    println!("{}", vec![1, 1, 2, 3, 3, 3, 1].into_iter().dedup_adjacent().debug_all());
    println!("{}", "aabbbcdd".chars().dedup_adjacent().collect::<String>());
    let combo = Squares { n: 0 }.pairwise().map(|(a, b)| b - a).every_other().take(3).debug_all();
    println!("combo {}", combo);
    println!("{}", Vec::<u8>::new().into_iter().pairwise().debug_all().is_empty());
}
