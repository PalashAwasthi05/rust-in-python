// The parts of Rust's standard library this interpreter provides, written in Rust.
// Body-less functions are intrinsics implemented in rust_in_python.py.

pub mod prelude {
    pub use crate::iter::Iterator;
    pub use crate::iter::IntoIterator;
    pub use crate::cmp::{Ordering, PartialEq, Eq, PartialOrd, Ord};
    pub use crate::mem::drop;
    pub use crate::convert::{From, Into, TryFrom, TryInto, AsRef, AsMut};
    pub use crate::string::ToString;
}

pub mod intrinsics {
    pub trait Concat {}
    impl<T> Concat for Vec<T> {}
    impl<T> Concat for [T] {}
    pub trait Separator {}
    impl Separator for &str {}
    impl Separator for char {}
    impl Separator for &String {}
    impl<T> Separator for &[T] {}
    impl<T> Separator for &Vec<T> {}
    pub trait Join { type Output; }
    impl Join for String { type Output = String; }
    impl Join for &str { type Output = String; }
    impl Join for &String { type Output = String; }
    impl<T> Join for Vec<T> { type Output = Vec<T>; }

    pub fn iter_max<I: crate::iter::Iterator>(iter: I, last: bool) -> Option<I::Item>;
    pub fn iter_collect<I: crate::iter::Iterator>(iter: I) -> Vec<I::Item>;
    pub fn range_next<A>(range: &mut crate::ops::Range<A>) -> Option<A>;
    pub fn range_inclusive_next<A>(range: &mut crate::ops::RangeInclusive<A>) -> Option<A>;
    pub fn range_from_next<A>(range: &mut crate::ops::RangeFrom<A>) -> Option<A>;
    pub fn compare<T>(left: &T, right: &T) -> i32;
    pub fn parse_int<T>(text: &str) -> Result<T, crate::num::ParseIntError>;
    pub fn try_integer<S, T>(value: S) -> Result<T, crate::num::TryFromIntError>;
    pub fn map_new<M>() -> M;
    pub fn map_len<M>(map: &M) -> usize;
    pub fn map_clear<M>(map: &mut M);
    pub fn map_insert<M, K, V>(map: &mut M, key: K, value: V) -> Option<V>;
    pub fn map_get<M, Q: ?Sized, R>(map: &M, key: &Q) -> R;
    pub fn map_get_key_value<M, Q: ?Sized, R>(map: &M, key: &Q) -> R;
    pub fn map_has<M, Q: ?Sized>(map: &M, key: &Q) -> bool;
    pub fn map_remove<M, Q: ?Sized, V>(map: &mut M, key: &Q) -> Option<V>;
    pub fn map_remove_entry<M, Q: ?Sized, R>(map: &mut M, key: &Q) -> R;
    pub fn map_or_insert<'a, M, K, V>(map: &'a mut M, key: K, value: V) -> &'a mut V;
    pub fn map_slot<'a, M, K, V>(map: &'a mut M, key: &K) -> &'a mut V;
    pub fn map_keys<M, R>(map: M) -> R;
    pub fn map_values<M, R>(map: M) -> R;
    pub fn map_items<M, R>(map: M) -> R;
    pub fn map_drain<M, R>(map: &mut M) -> R;
    pub fn map_retain<M, F>(map: &mut M, f: F);
    pub fn map_extend<M, I>(map: &mut M, iter: I);
    pub fn map_end<M, R>(map: &M, last: bool) -> R;
    pub fn map_pop<M, R>(map: &mut M, last: bool) -> R;
    pub fn map_range<M, B, R>(map: &M, range: B) -> R;
    pub fn map_debug<M>(map: &M, f: &mut crate::fmt::Formatter) -> crate::fmt::Result;
    pub fn set_insert<S, T>(set: &mut S, value: T) -> bool;
    pub fn set_remove<S, Q: ?Sized>(set: &mut S, value: &Q) -> bool;
    pub fn set_get<S, Q: ?Sized, R>(set: &S, value: &Q) -> R;
    pub fn set_take<S, Q: ?Sized, R>(set: &mut S, value: &Q) -> R;
    pub fn set_drain<S, R>(set: &mut S) -> R;
    pub fn set_retain<S, F>(set: &mut S, f: F);
    pub fn set_extend<S, I>(set: &mut S, iter: I);
    pub fn set_combine<S, R>(set: &S, other: &S, mode: i32) -> R;
    pub fn set_relation<S>(set: &S, other: &S, mode: i32) -> bool;
    pub fn set_end<S, R>(set: &S, last: bool) -> R;
    pub fn set_pop<S, R>(set: &mut S, last: bool) -> R;
    pub fn set_range<S, B, R>(set: &S, range: B) -> R;
    pub fn lazy_elements<C, R>(items: C) -> R;
    pub fn lazy_range<C, B, R>(items: C, range: B) -> R;
    pub fn parse_float<T>(text: &str) -> Result<T, crate::num::ParseFloatError>;
    pub fn lazy_next<T>(iter: &mut crate::iter::Lazy<T>) -> Option<T>;
}

pub mod iter {
    pub trait Iterator {
        type Item;
        fn next(&mut self) -> Option<Self::Item>;

        fn count(self) -> usize where Self: Sized {
            let mut iter = self;
            let mut total = 0;
            while let Some(_) = iter.next() { total += 1; }
            total
        }
        fn last(self) -> Option<Self::Item> where Self: Sized {
            let mut iter = self;
            let mut last = None;
            while let Some(item) = iter.next() { last = Some(item); }
            last
        }
        fn nth(&mut self, n: usize) -> Option<Self::Item> {
            let mut remaining = n;
            while let Some(item) = self.next() {
                if remaining == 0 { return Some(item); }
                remaining -= 1;
            }
            None
        }
        fn step_by(self, step: usize) -> StepBy<Self> where Self: Sized {
            if step == 0 { panic!("assertion failed: step != 0"); }
            StepBy { iter: self, step: step - 1, first: true }
        }
        fn chain<U>(self, other: U) -> Chain<Self, U::IntoIter> where Self: Sized, U: IntoIterator {
            Chain { first: Some(self), second: other.into_iter() }
        }
        fn zip<U>(self, other: U) -> Zip<Self, U::IntoIter> where Self: Sized, U: IntoIterator {
            Zip { a: self, b: other.into_iter() }
        }
        fn map<B, F>(self, f: F) -> Map<Self, F> where Self: Sized, F: FnMut(Self::Item) -> B {
            Map { iter: self, f: f }
        }
        fn for_each<F>(self, f: F) where Self: Sized, F: FnMut(Self::Item) {
            let mut iter = self;
            let mut f = f;
            while let Some(item) = iter.next() { f(item); }
        }
        fn filter<P>(self, predicate: P) -> Filter<Self, P> where Self: Sized, P: FnMut(&Self::Item) -> bool {
            Filter { iter: self, predicate: predicate }
        }
        fn filter_map<B, F>(self, f: F) -> FilterMap<Self, F> where Self: Sized, F: FnMut(Self::Item) -> Option<B> {
            FilterMap { iter: self, f: f }
        }
        fn enumerate(self) -> Enumerate<Self> where Self: Sized {
            Enumerate { iter: self, count: 0 }
        }
        fn peekable(self) -> Peekable<Self> where Self: Sized {
            Peekable { iter: self, peeked: None }
        }
        fn skip_while<P>(self, predicate: P) -> SkipWhile<Self, P> where Self: Sized, P: FnMut(&Self::Item) -> bool {
            SkipWhile { iter: self, done: false, predicate: predicate }
        }
        fn take_while<P>(self, predicate: P) -> TakeWhile<Self, P> where Self: Sized, P: FnMut(&Self::Item) -> bool {
            TakeWhile { iter: self, done: false, predicate: predicate }
        }
        fn map_while<B, P>(self, predicate: P) -> MapWhile<Self, P> where Self: Sized, P: FnMut(Self::Item) -> Option<B> {
            MapWhile { iter: self, done: false, predicate: predicate }
        }
        fn skip(self, n: usize) -> Skip<Self> where Self: Sized {
            Skip { iter: self, n: n }
        }
        fn take(self, n: usize) -> Take<Self> where Self: Sized {
            Take { iter: self, n: n }
        }
        fn scan<St, B, F>(self, initial_state: St, f: F) -> Scan<Self, St, F> where Self: Sized, F: FnMut(&mut St, Self::Item) -> Option<B> {
            Scan { iter: self, f: f, state: initial_state }
        }
        fn flat_map<U, F>(self, f: F) -> FlatMap<Self, U, F> where Self: Sized, U: IntoIterator, F: FnMut(Self::Item) -> U {
            FlatMap { iter: self, f: f, front: None }
        }
        fn fuse(self) -> Fuse<Self> where Self: Sized {
            Fuse { iter: Some(self) }
        }
        fn inspect<F>(self, f: F) -> Inspect<Self, F> where Self: Sized, F: FnMut(&Self::Item) {
            Inspect { iter: self, f: f }
        }
        fn by_ref(&mut self) -> &mut Self where Self: Sized {
            self
        }
        fn fold<B, F>(self, init: B, f: F) -> B where Self: Sized, F: FnMut(B, Self::Item) -> B {
            let mut iter = self;
            let mut f = f;
            let mut accumulator = init;
            while let Some(item) = iter.next() { accumulator = f(accumulator, item); }
            accumulator
        }
        fn reduce<F>(self, f: F) -> Option<Self::Item> where Self: Sized, F: FnMut(Self::Item, Self::Item) -> Self::Item {
            let mut iter = self;
            let mut f = f;
            let mut accumulator = iter.next()?;
            while let Some(item) = iter.next() { accumulator = f(accumulator, item); }
            Some(accumulator)
        }
        fn all<F>(&mut self, f: F) -> bool where Self: Sized, F: FnMut(Self::Item) -> bool {
            let mut f = f;
            while let Some(item) = self.next() {
                if !f(item) { return false; }
            }
            true
        }
        fn any<F>(&mut self, f: F) -> bool where Self: Sized, F: FnMut(Self::Item) -> bool {
            let mut f = f;
            while let Some(item) = self.next() {
                if f(item) { return true; }
            }
            false
        }
        fn find<P>(&mut self, predicate: P) -> Option<Self::Item> where Self: Sized, P: FnMut(&Self::Item) -> bool {
            let mut predicate = predicate;
            while let Some(item) = self.next() {
                if predicate(&item) { return Some(item); }
            }
            None
        }
        fn find_map<B, F>(&mut self, f: F) -> Option<B> where Self: Sized, F: FnMut(Self::Item) -> Option<B> {
            let mut f = f;
            while let Some(item) = self.next() {
                if let Some(found) = f(item) { return Some(found); }
            }
            None
        }
        fn position<P>(&mut self, predicate: P) -> Option<usize> where Self: Sized, P: FnMut(Self::Item) -> bool {
            let mut predicate = predicate;
            let mut index = 0;
            while let Some(item) = self.next() {
                if predicate(item) { return Some(index); }
                index += 1;
            }
            None
        }
        fn max(self) -> Option<Self::Item> where Self: Sized {
            crate::intrinsics::iter_max(self, true)
        }
        fn min(self) -> Option<Self::Item> where Self: Sized {
            crate::intrinsics::iter_max(self, false)
        }
        fn max_by_key<B, F>(self, f: F) -> Option<Self::Item> where Self: Sized, F: FnMut(&Self::Item) -> B {
            let mut iter = self;
            let mut f = f;
            let mut best = iter.next()?;
            let mut best_key = f(&best);
            while let Some(item) = iter.next() {
                let key = f(&item);
                if crate::intrinsics::compare(&key, &best_key) >= 0 { best = item; best_key = key; }
            }
            Some(best)
        }
        fn min_by_key<B, F>(self, f: F) -> Option<Self::Item> where Self: Sized, F: FnMut(&Self::Item) -> B {
            let mut iter = self;
            let mut f = f;
            let mut best = iter.next()?;
            let mut best_key = f(&best);
            while let Some(item) = iter.next() {
                let key = f(&item);
                if crate::intrinsics::compare(&key, &best_key) < 0 { best = item; best_key = key; }
            }
            Some(best)
        }
        fn max_by<F>(self, compare: F) -> Option<Self::Item> where Self: Sized, F: FnMut(&Self::Item, &Self::Item) -> crate::cmp::Ordering {
            let mut iter = self;
            let mut compare = compare;
            let mut best = iter.next()?;
            while let Some(item) = iter.next() {
                if compare(&item, &best) != crate::cmp::Ordering::Less { best = item; }
            }
            Some(best)
        }
        fn min_by<F>(self, compare: F) -> Option<Self::Item> where Self: Sized, F: FnMut(&Self::Item, &Self::Item) -> crate::cmp::Ordering {
            let mut iter = self;
            let mut compare = compare;
            let mut best = iter.next()?;
            while let Some(item) = iter.next() {
                if compare(&item, &best) == crate::cmp::Ordering::Less { best = item; }
            }
            Some(best)
        }
        fn rev(self) -> Rev<Self::Item> where Self: Sized {
            Rev { items: crate::intrinsics::iter_collect(self) }
        }
        fn partition<B, F>(self, f: F) -> (Vec<Self::Item>, Vec<Self::Item>) where Self: Sized, F: FnMut(&Self::Item) -> bool {
            let mut iter = self;
            let mut f = f;
            let mut left = Vec::new();
            let mut right = Vec::new();
            while let Some(item) = iter.next() {
                if f(&item) { left.push(item); } else { right.push(item); }
            }
            (left, right)
        }
    }

    pub trait IntoIterator {
        type IntoIter: Iterator;
        fn into_iter(self) -> Self::IntoIter;
    }

    pub struct StepBy<I> { iter: I, step: usize, first: bool }
    impl<I: Iterator> Iterator for StepBy<I> {
        type Item = I::Item;
        fn next(&mut self) -> Option<I::Item> {
            if self.first {
                self.first = false;
                return self.iter.next();
            }
            self.iter.nth(self.step)
        }
    }

    pub struct Chain<A, B> { first: Option<A>, second: B }
    impl<A: Iterator, B: Iterator<Item = A::Item>> Iterator for Chain<A, B> {
        type Item = A::Item;
        fn next(&mut self) -> Option<A::Item> {
            if let Some(first) = &mut self.first {
                match first.next() {
                    Some(item) => return Some(item),
                    None => {}
                }
                self.first = None;
            }
            self.second.next()
        }
    }

    pub struct Zip<A, B> { a: A, b: B }
    impl<A: Iterator, B: Iterator> Iterator for Zip<A, B> {
        type Item = (A::Item, B::Item);
        fn next(&mut self) -> Option<(A::Item, B::Item)> {
            let left = self.a.next()?;
            let right = self.b.next()?;
            Some((left, right))
        }
    }

    pub struct Map<I, F> { iter: I, f: F }
    impl<B, I: Iterator, F: FnMut(I::Item) -> B> Iterator for Map<I, F> {
        type Item = B;
        fn next(&mut self) -> Option<B> {
            match self.iter.next() {
                Some(item) => Some((self.f)(item)),
                None => None,
            }
        }
    }

    pub struct Filter<I, P> { iter: I, predicate: P }
    impl<I: Iterator, P: FnMut(&I::Item) -> bool> Iterator for Filter<I, P> {
        type Item = I::Item;
        fn next(&mut self) -> Option<I::Item> {
            while let Some(item) = self.iter.next() {
                if (self.predicate)(&item) { return Some(item); }
            }
            None
        }
    }

    pub struct FilterMap<I, F> { iter: I, f: F }
    impl<B, I: Iterator, F: FnMut(I::Item) -> Option<B>> Iterator for FilterMap<I, F> {
        type Item = B;
        fn next(&mut self) -> Option<B> {
            while let Some(item) = self.iter.next() {
                if let Some(mapped) = (self.f)(item) { return Some(mapped); }
            }
            None
        }
    }

    pub struct Enumerate<I> { iter: I, count: usize }
    impl<I: Iterator> Iterator for Enumerate<I> {
        type Item = (usize, I::Item);
        fn next(&mut self) -> Option<(usize, I::Item)> {
            let item = self.iter.next()?;
            let index = self.count;
            self.count += 1;
            Some((index, item))
        }
    }

    pub struct Peekable<I: Iterator> { iter: I, peeked: Option<Option<I::Item>> }
    impl<I: Iterator> Iterator for Peekable<I> {
        type Item = I::Item;
        fn next(&mut self) -> Option<I::Item> {
            match self.peeked.take() {
                Some(item) => item,
                None => self.iter.next(),
            }
        }
    }
    impl<I: Iterator> Peekable<I> {
        pub fn peek(&mut self) -> Option<&I::Item> {
            if self.peeked.is_none() {
                self.peeked = Some(self.iter.next());
            }
            match &self.peeked {
                Some(Some(item)) => Some(item),
                _ => None,
            }
        }
        pub fn next_if<F: FnOnce(&I::Item) -> bool>(&mut self, f: F) -> Option<I::Item> {
            let item = self.next()?;
            if f(&item) { return Some(item); }
            self.peeked = Some(Some(item));
            None
        }
    }

    pub struct SkipWhile<I, P> { iter: I, done: bool, predicate: P }
    impl<I: Iterator, P: FnMut(&I::Item) -> bool> Iterator for SkipWhile<I, P> {
        type Item = I::Item;
        fn next(&mut self) -> Option<I::Item> {
            while let Some(item) = self.iter.next() {
                if self.done || !(self.predicate)(&item) {
                    self.done = true;
                    return Some(item);
                }
            }
            None
        }
    }

    pub struct TakeWhile<I, P> { iter: I, done: bool, predicate: P }
    impl<I: Iterator, P: FnMut(&I::Item) -> bool> Iterator for TakeWhile<I, P> {
        type Item = I::Item;
        fn next(&mut self) -> Option<I::Item> {
            if self.done { return None; }
            let item = self.iter.next()?;
            if (self.predicate)(&item) { return Some(item); }
            self.done = true;
            None
        }
    }

    pub struct MapWhile<I, P> { iter: I, done: bool, predicate: P }
    impl<B, I: Iterator, P: FnMut(I::Item) -> Option<B>> Iterator for MapWhile<I, P> {
        type Item = B;
        fn next(&mut self) -> Option<B> {
            if self.done { return None; }
            let item = self.iter.next()?;
            let mapped = (self.predicate)(item);
            if mapped.is_none() { self.done = true; }
            mapped
        }
    }

    pub struct Skip<I> { iter: I, n: usize }
    impl<I: Iterator> Iterator for Skip<I> {
        type Item = I::Item;
        fn next(&mut self) -> Option<I::Item> {
            while self.n > 0 {
                self.n -= 1;
                self.iter.next()?;
            }
            self.iter.next()
        }
    }

    pub struct Take<I> { iter: I, n: usize }
    impl<I: Iterator> Iterator for Take<I> {
        type Item = I::Item;
        fn next(&mut self) -> Option<I::Item> {
            if self.n == 0 { return None; }
            self.n -= 1;
            self.iter.next()
        }
    }

    pub struct Scan<I, St, F> { iter: I, f: F, state: St }
    impl<B, I: Iterator, St, F: FnMut(&mut St, I::Item) -> Option<B>> Iterator for Scan<I, St, F> {
        type Item = B;
        fn next(&mut self) -> Option<B> {
            let item = self.iter.next()?;
            (self.f)(&mut self.state, item)
        }
    }

    pub struct FlatMap<I, U: IntoIterator, F> { iter: I, f: F, front: Option<U::IntoIter> }
    impl<I: Iterator, U: IntoIterator, F: FnMut(I::Item) -> U> Iterator for FlatMap<I, U, F> {
        type Item = <U::IntoIter as Iterator>::Item;
        fn next(&mut self) -> Option<<U::IntoIter as Iterator>::Item> {
            loop {
                if let Some(inner) = &mut self.front {
                    if let Some(item) = inner.next() { return Some(item); }
                }
                let next = self.iter.next()?;
                self.front = Some((self.f)(next).into_iter());
            }
        }
    }

    pub struct Lazy<T> { items: Vec<T> }
    impl<T> Iterator for Lazy<T> {
        type Item = T;
        fn next(&mut self) -> Option<T> { crate::intrinsics::lazy_next(self) }
    }

    pub struct Fuse<I> { iter: Option<I> }
    impl<I: Iterator> Iterator for Fuse<I> {
        type Item = I::Item;
        fn next(&mut self) -> Option<I::Item> {
            let item = match &mut self.iter {
                Some(iter) => iter.next(),
                None => return None,
            };
            if item.is_none() { self.iter = None; }
            item
        }
    }

    pub struct Inspect<I, F> { iter: I, f: F }
    impl<I: Iterator, F: FnMut(&I::Item)> Iterator for Inspect<I, F> {
        type Item = I::Item;
        fn next(&mut self) -> Option<I::Item> {
            let item = self.iter.next()?;
            (self.f)(&item);
            Some(item)
        }
    }

    pub struct Rev<T> { items: Vec<T> }
    impl<T> Iterator for Rev<T> {
        type Item = T;
        fn next(&mut self) -> Option<T> { self.items.pop() }
    }

    pub struct Empty<T> { items: Vec<T> }
    impl<T> Iterator for Empty<T> {
        type Item = T;
        fn next(&mut self) -> Option<T> { None }
    }
    pub fn empty<T>() -> Empty<T> { Empty { items: Vec::new() } }

    pub struct Once<T> { item: Option<T> }
    impl<T> Iterator for Once<T> {
        type Item = T;
        fn next(&mut self) -> Option<T> { self.item.take() }
    }
    pub fn once<T>(value: T) -> Once<T> { Once { item: Some(value) } }

    pub struct Repeat<T> { item: T }
    impl<T: Clone> Iterator for Repeat<T> {
        type Item = T;
        fn next(&mut self) -> Option<T> { Some(self.item.clone()) }
    }
    pub fn repeat<T: Clone>(value: T) -> Repeat<T> { Repeat { item: value } }

    pub struct FromFn<F> { f: F }
    impl<T, F: FnMut() -> Option<T>> Iterator for FromFn<F> {
        type Item = T;
        fn next(&mut self) -> Option<T> { (self.f)() }
    }
    pub fn from_fn<T, F: FnMut() -> Option<T>>(f: F) -> FromFn<F> { FromFn { f: f } }

    pub struct Successors<T, F> { next: Option<T>, f: F }
    impl<T, F: FnMut(&T) -> Option<T>> Iterator for Successors<T, F> {
        type Item = T;
        fn next(&mut self) -> Option<T> {
            let item = self.next.take()?;
            self.next = (self.f)(&item);
            Some(item)
        }
    }
    pub fn successors<T, F: FnMut(&T) -> Option<T>>(first: Option<T>, succ: F) -> Successors<T, F> {
        Successors { next: first, f: succ }
    }
}

pub mod cmp {
    #[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord, Hash)]
    pub enum Ordering { Less, Equal, Greater }

    impl Ordering {
        pub fn is_eq(self) -> bool { self == Ordering::Equal }
        pub fn is_ne(self) -> bool { self != Ordering::Equal }
        pub fn is_lt(self) -> bool { self == Ordering::Less }
        pub fn is_gt(self) -> bool { self == Ordering::Greater }
        pub fn is_le(self) -> bool { self != Ordering::Greater }
        pub fn is_ge(self) -> bool { self != Ordering::Less }
        pub fn reverse(self) -> Ordering {
            match self { Ordering::Less => Ordering::Greater, Ordering::Equal => Ordering::Equal, Ordering::Greater => Ordering::Less }
        }
        pub fn then(self, other: Ordering) -> Ordering {
            match self { Ordering::Equal => other, _ => self }
        }
        pub fn then_with<F: FnOnce() -> Ordering>(self, f: F) -> Ordering {
            match self { Ordering::Equal => f(), _ => self }
        }
    }

    pub trait PartialEq {
        fn eq(&self, other: &Self) -> bool;
        fn ne(&self, other: &Self) -> bool { !self.eq(other) }
    }

    pub trait Eq: PartialEq {}

    pub trait PartialOrd: PartialEq {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering>;
        fn lt(&self, other: &Self) -> bool { self.partial_cmp(other) == Some(Ordering::Less) }
        fn le(&self, other: &Self) -> bool {
            match self.partial_cmp(other) { Some(Ordering::Less) | Some(Ordering::Equal) => true, _ => false }
        }
        fn gt(&self, other: &Self) -> bool { self.partial_cmp(other) == Some(Ordering::Greater) }
        fn ge(&self, other: &Self) -> bool {
            match self.partial_cmp(other) { Some(Ordering::Greater) | Some(Ordering::Equal) => true, _ => false }
        }
    }

    pub trait Ord: Eq + PartialOrd {
        fn cmp(&self, other: &Self) -> Ordering;
        fn max(self, other: Self) -> Self where Self: Sized {
            if other.cmp(&self) == Ordering::Less { self } else { other }
        }
        fn min(self, other: Self) -> Self where Self: Sized {
            if other.cmp(&self) == Ordering::Less { other } else { self }
        }
        fn clamp(self, min: Self, max: Self) -> Self where Self: Sized {
            if self.cmp(&min) == Ordering::Less { min } else if self.cmp(&max) == Ordering::Greater { max } else { self }
        }
    }

    pub fn max<T: Ord>(v1: T, v2: T) -> T { v1.max(v2) }
    pub fn min<T: Ord>(v1: T, v2: T) -> T { v1.min(v2) }
    pub fn max_by_key<T, K: Ord, F: FnMut(&T) -> K>(v1: T, v2: T, f: F) -> T {
        let mut f = f;
        if f(&v2).cmp(&f(&v1)) == Ordering::Less { v1 } else { v2 }
    }
    pub fn min_by_key<T, K: Ord, F: FnMut(&T) -> K>(v1: T, v2: T, f: F) -> T {
        let mut f = f;
        if f(&v2).cmp(&f(&v1)) == Ordering::Less { v2 } else { v1 }
    }

    #[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
    pub struct Reverse<T>(pub T);
    impl<T: PartialOrd> PartialOrd for Reverse<T> {
        fn partial_cmp(&self, other: &Reverse<T>) -> Option<Ordering> { other.0.partial_cmp(&self.0) }
    }
    impl<T: Ord> Ord for Reverse<T> {
        fn cmp(&self, other: &Reverse<T>) -> Ordering { other.0.cmp(&self.0) }
    }
}

pub mod slice {
    use crate::iter::Lazy;
    use crate::cmp::Ordering;

    impl<T> [T] {
        pub fn first(&self) -> Option<&T>;
        pub fn last(&self) -> Option<&T>;
        pub fn first_mut(&mut self) -> Option<&mut T>;
        pub fn last_mut(&mut self) -> Option<&mut T>;
        pub fn get(&self, index: usize) -> Option<&T>;
        pub fn get_mut(&mut self, index: usize) -> Option<&mut T>;
        pub fn split_first(&self) -> Option<(&T, &[T])>;
        pub fn split_last(&self) -> Option<(&T, &[T])>;
        pub fn split_at(&self, mid: usize) -> (&[T], &[T]);
        pub fn contains(&self, x: &T) -> bool where T: PartialEq;
        pub fn starts_with(&self, needle: &[T]) -> bool where T: PartialEq;
        pub fn ends_with(&self, needle: &[T]) -> bool where T: PartialEq;
        pub fn swap(&mut self, a: usize, b: usize);
        pub fn reverse(&mut self);
        pub fn sort(&mut self) where T: Ord;
        pub fn sort_unstable(&mut self) where T: Ord;
        pub fn sort_by<F: FnMut(&T, &T) -> Ordering>(&mut self, compare: F);
        pub fn sort_unstable_by<F: FnMut(&T, &T) -> Ordering>(&mut self, compare: F);
        pub fn sort_by_key<K: Ord, F: FnMut(&T) -> K>(&mut self, f: F);
        pub fn sort_unstable_by_key<K: Ord, F: FnMut(&T) -> K>(&mut self, f: F);
        pub fn sort_by_cached_key<K: Ord, F: FnMut(&T) -> K>(&mut self, f: F);
        pub fn binary_search(&self, x: &T) -> Result<usize, usize> where T: Ord;
        pub fn binary_search_by<F: FnMut(&T) -> Ordering>(&self, f: F) -> Result<usize, usize>;
        pub fn binary_search_by_key<B: Ord, F: FnMut(&T) -> B>(&self, b: &B, f: F) -> Result<usize, usize>;
        pub fn partition_point<P: FnMut(&T) -> bool>(&self, pred: P) -> usize;
        pub fn to_vec(&self) -> Vec<T> where T: Clone;
        pub fn into_vec(&self) -> Vec<T> where T: Clone;
        pub fn repeat(&self, n: usize) -> Vec<T> where T: Clone;
        pub fn fill(&mut self, value: T) where T: Clone;
        pub fn rotate_left(&mut self, mid: usize);
        pub fn rotate_right(&mut self, k: usize);
        pub fn copy_from_slice(&mut self, src: &[T]) where T: Copy;
        pub fn clone_from_slice(&mut self, src: &[T]) where T: Clone;
        pub fn windows(&self, size: usize) -> Lazy<&[T]>;
        pub fn chunks(&self, size: usize) -> Lazy<&[T]>;
        pub fn chunks_exact(&self, size: usize) -> Lazy<&[T]>;
        pub fn rchunks(&self, size: usize) -> Lazy<&[T]>;
        pub fn split<F: FnMut(&T) -> bool>(&self, pred: F) -> Lazy<&[T]>;
        pub fn iter_positions(&self) -> usize { self.len() }
        pub fn concat(&self) -> <T as crate::intrinsics::Join>::Output where T: crate::intrinsics::Join;
        pub fn join<S: crate::intrinsics::Separator>(&self, separator: S) -> <T as crate::intrinsics::Join>::Output where T: crate::intrinsics::Join;
        pub fn max_len(&self) -> usize { self.len() }
    }
}

pub mod vec {
    use crate::iter::Lazy;

    impl<T> Vec<T> {
        pub fn with_capacity(capacity: usize) -> Vec<T> { Vec::new() }
        pub fn capacity(&self) -> usize { self.len() }
        pub fn reserve(&mut self, additional: usize) {}
        pub fn shrink_to_fit(&mut self) {}
        pub fn insert(&mut self, index: usize, element: T);
        pub fn remove(&mut self, index: usize) -> T;
        pub fn swap_remove(&mut self, index: usize) -> T;
        pub fn append(&mut self, other: &mut Vec<T>);
        pub fn extend_from_slice(&mut self, other: &[T]) where T: Clone;
        pub fn extend<I: IntoIterator>(&mut self, iter: I);
        pub fn retain<F: FnMut(&T) -> bool>(&mut self, f: F);
        pub fn retain_mut<F: FnMut(&mut T) -> bool>(&mut self, f: F);
        pub fn dedup(&mut self) where T: PartialEq;
        pub fn dedup_by_key<K: PartialEq, F: FnMut(&mut T) -> K>(&mut self, key: F);
        pub fn dedup_by<F: FnMut(&mut T, &mut T) -> bool>(&mut self, same_bucket: F);
        pub fn split_off(&mut self, at: usize) -> Vec<T>;
        pub fn resize(&mut self, new_len: usize, value: T) where T: Clone;
        pub fn resize_with<F: FnMut() -> T>(&mut self, new_len: usize, f: F);
        pub fn drain<R: crate::ops::RangeBounds>(&mut self, range: R) -> Lazy<T>;
        pub fn into_boxed_slice(self) -> Vec<T> { self }
        pub fn from_elem(elem: T, n: usize) -> Vec<T> where T: Clone { vec![elem; n] }
        pub fn from_iter<I: IntoIterator>(iter: I) -> Vec<T> {
            let mut items = Vec::new();
            items.extend(iter);
            items
        }
    }
}

pub mod mem {
    pub fn replace<T>(dest: &mut T, src: T) -> T;
    pub fn swap<T>(x: &mut T, y: &mut T);
    pub fn take<T: Default>(dest: &mut T) -> T { replace(dest, T::default()) }
    pub fn drop<T>(_value: T) {}
}

pub mod option {
    impl<T> Option<T> {
        pub fn is_some_and<F: FnOnce(T) -> bool>(self, f: F) -> bool {
            match self { Some(value) => f(value), None => false }
        }
        pub fn is_none_or<F: FnOnce(T) -> bool>(self, f: F) -> bool {
            match self { Some(value) => f(value), None => true }
        }
        pub fn as_ref(&self) -> Option<&T> {
            match self { Some(value) => Some(value), None => None }
        }
        pub fn as_mut(&mut self) -> Option<&mut T> {
            match self { Some(value) => Some(value), None => None }
        }
        pub fn unwrap_or_else<F: FnOnce() -> T>(self, f: F) -> T {
            match self { Some(value) => value, None => f() }
        }
        pub fn unwrap_or_default(self) -> T where T: Default {
            match self { Some(value) => value, None => T::default() }
        }
        pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Option<U> {
            match self { Some(value) => Some(f(value)), None => None }
        }
        pub fn inspect<F: FnOnce(&T)>(self, f: F) -> Option<T> {
            if let Some(value) = &self { f(value); }
            self
        }
        pub fn map_or<U, F: FnOnce(T) -> U>(self, default: U, f: F) -> U {
            match self { Some(value) => f(value), None => default }
        }
        pub fn map_or_else<U, D: FnOnce() -> U, F: FnOnce(T) -> U>(self, default: D, f: F) -> U {
            match self { Some(value) => f(value), None => default() }
        }
        pub fn ok_or<E>(self, err: E) -> Result<T, E> {
            match self { Some(value) => Ok(value), None => Err(err) }
        }
        pub fn ok_or_else<E, F: FnOnce() -> E>(self, err: F) -> Result<T, E> {
            match self { Some(value) => Ok(value), None => Err(err()) }
        }
        pub fn and<U>(self, optb: Option<U>) -> Option<U> {
            match self { Some(_) => optb, None => None }
        }
        pub fn and_then<U, F: FnOnce(T) -> Option<U>>(self, f: F) -> Option<U> {
            match self { Some(value) => f(value), None => None }
        }
        pub fn filter<P: FnOnce(&T) -> bool>(self, predicate: P) -> Option<T> {
            if let Some(value) = self {
                if predicate(&value) { return Some(value); }
            }
            None
        }
        pub fn or(self, optb: Option<T>) -> Option<T> {
            match self { Some(value) => Some(value), None => optb }
        }
        pub fn or_else<F: FnOnce() -> Option<T>>(self, f: F) -> Option<T> {
            match self { Some(value) => Some(value), None => f() }
        }
        pub fn xor(self, optb: Option<T>) -> Option<T> {
            match (self, optb) {
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                _ => None,
            }
        }
        pub fn insert(&mut self, value: T) -> &mut T {
            *self = Some(value);
            match self { Some(value) => value, None => panic!("unreachable") }
        }
        pub fn get_or_insert(&mut self, value: T) -> &mut T {
            if self.is_none() { *self = Some(value); }
            match self { Some(value) => value, None => panic!("unreachable") }
        }
        pub fn get_or_insert_with<F: FnOnce() -> T>(&mut self, f: F) -> &mut T {
            if self.is_none() { *self = Some(f()); }
            match self { Some(value) => value, None => panic!("unreachable") }
        }
        pub fn take(&mut self) -> Option<T> {
            crate::mem::replace(self, None)
        }
        pub fn take_if<P: FnOnce(&mut T) -> bool>(&mut self, predicate: P) -> Option<T> {
            let taken = match self { Some(value) => predicate(value), None => false };
            if taken { self.take() } else { None }
        }
        pub fn replace(&mut self, value: T) -> Option<T> {
            crate::mem::replace(self, Some(value))
        }
        pub fn zip<U>(self, other: Option<U>) -> Option<(T, U)> {
            match (self, other) {
                (Some(a), Some(b)) => Some((a, b)),
                _ => None,
            }
        }
        pub fn unwrap_unchecked(self) -> T { self.unwrap() }
    }

    impl<T: Clone> Option<&T> {
        pub fn cloned(self) -> Option<T> {
            match self { Some(value) => Some(value.clone()), None => None }
        }
        pub fn copied(self) -> Option<T> {
            match self { Some(value) => Some(value.clone()), None => None }
        }
    }

    impl<T: Clone> Option<&mut T> {
        pub fn cloned(self) -> Option<T> {
            match self { Some(value) => Some(value.clone()), None => None }
        }
        pub fn copied(self) -> Option<T> {
            match self { Some(value) => Some(value.clone()), None => None }
        }
    }

    impl<T> Option<Option<T>> {
        pub fn flatten(self) -> Option<T> {
            match self { Some(inner) => inner, None => None }
        }
    }

    impl<T, E> Option<Result<T, E>> {
        pub fn transpose(self) -> Result<Option<T>, E> {
            match self {
                Some(Ok(value)) => Ok(Some(value)),
                Some(Err(err)) => Err(err),
                None => Ok(None),
            }
        }
    }
}

pub mod result {
    impl<T, E> Result<T, E> {
        pub fn is_ok_and<F: FnOnce(T) -> bool>(self, f: F) -> bool {
            match self { Ok(value) => f(value), Err(_) => false }
        }
        pub fn is_err_and<F: FnOnce(E) -> bool>(self, f: F) -> bool {
            match self { Ok(_) => false, Err(err) => f(err) }
        }
        pub fn ok(self) -> Option<T> {
            match self { Ok(value) => Some(value), Err(_) => None }
        }
        pub fn err(self) -> Option<E> {
            match self { Ok(_) => None, Err(err) => Some(err) }
        }
        pub fn as_ref(&self) -> Result<&T, &E> {
            match self { Ok(value) => Ok(value), Err(err) => Err(err) }
        }
        pub fn as_mut(&mut self) -> Result<&mut T, &mut E> {
            match self { Ok(value) => Ok(value), Err(err) => Err(err) }
        }
        pub fn map<U, F: FnOnce(T) -> U>(self, op: F) -> Result<U, E> {
            match self { Ok(value) => Ok(op(value)), Err(err) => Err(err) }
        }
        pub fn map_or<U, F: FnOnce(T) -> U>(self, default: U, f: F) -> U {
            match self { Ok(value) => f(value), Err(_) => default }
        }
        pub fn map_or_else<U, D: FnOnce(E) -> U, F: FnOnce(T) -> U>(self, default: D, f: F) -> U {
            match self { Ok(value) => f(value), Err(err) => default(err) }
        }
        pub fn map_err<F, O: FnOnce(E) -> F>(self, op: O) -> Result<T, F> {
            match self { Ok(value) => Ok(value), Err(err) => Err(op(err)) }
        }
        pub fn inspect<F: FnOnce(&T)>(self, f: F) -> Result<T, E> {
            if let Ok(value) = &self { f(value); }
            self
        }
        pub fn inspect_err<F: FnOnce(&E)>(self, f: F) -> Result<T, E> {
            if let Err(err) = &self { f(err); }
            self
        }
        pub fn and<U>(self, res: Result<U, E>) -> Result<U, E> {
            match self { Ok(_) => res, Err(err) => Err(err) }
        }
        pub fn and_then<U, F: FnOnce(T) -> Result<U, E>>(self, op: F) -> Result<U, E> {
            match self { Ok(value) => op(value), Err(err) => Err(err) }
        }
        pub fn or<F>(self, res: Result<T, F>) -> Result<T, F> {
            match self { Ok(value) => Ok(value), Err(_) => res }
        }
        pub fn or_else<F, O: FnOnce(E) -> Result<T, F>>(self, op: O) -> Result<T, F> {
            match self { Ok(value) => Ok(value), Err(err) => op(err) }
        }
        pub fn unwrap_or_else<F: FnOnce(E) -> T>(self, op: F) -> T {
            match self { Ok(value) => value, Err(err) => op(err) }
        }
        pub fn unwrap_or_default(self) -> T where T: Default {
            match self { Ok(value) => value, Err(_) => T::default() }
        }
        pub fn unwrap_err(self) -> E where T: crate::fmt::Debug {
            match self {
                Ok(value) => panic!("called `Result::unwrap_err()` on an `Ok` value: {:?}", value),
                Err(err) => err,
            }
        }
        pub fn expect_err(self, msg: &str) -> E where T: crate::fmt::Debug {
            match self {
                Ok(value) => panic!("{}: {:?}", msg, value),
                Err(err) => err,
            }
        }
    }

    impl<T, E> Result<Option<T>, E> {
        pub fn transpose(self) -> Option<Result<T, E>> {
            match self {
                Ok(Some(value)) => Some(Ok(value)),
                Ok(None) => None,
                Err(err) => Some(Err(err)),
            }
        }
    }
}

pub mod fmt {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Error;

    pub type Result = core::result::Result<(), Error>;

    impl Display for Error {
        fn fmt(&self, f: &mut Formatter) -> Result { f.write_str("an error occurred when formatting an argument") }
    }

    pub struct Formatter { buf: String, alternate: bool }

    impl Formatter {
        pub fn write_str(&mut self, data: &str) -> Result {
            self.buf.push_str(data);
            Ok(())
        }
        pub fn write_fmt(&mut self, text: String) -> Result {
            self.buf.push_str(&text);
            Ok(())
        }
        pub fn write_char(&mut self, c: char) -> Result {
            self.buf.push(c);
            Ok(())
        }
        pub fn pad(&mut self, data: &str) -> Result { self.write_str(data) }
        pub fn alternate(&self) -> bool { self.alternate }
    }

    pub trait Display {
        fn fmt(&self, f: &mut Formatter) -> Result;
    }

    pub trait Debug {
        fn fmt(&self, f: &mut Formatter) -> Result;
    }

    pub trait Write {
        fn write_str(&mut self, s: &str) -> Result;
        fn write_char(&mut self, c: char) -> Result {
            let mut text = String::new();
            text.push(c);
            self.write_str(&text)
        }
        fn write_fmt(&mut self, text: String) -> Result {
            self.write_str(&text)
        }
    }

    impl Write for String {
        fn write_str(&mut self, s: &str) -> Result {
            self.push_str(s);
            Ok(())
        }
    }
}

pub mod clone {
}

pub mod default {
}

pub mod marker {
}

pub mod ops {
    use crate::fmt;

    pub trait Add<Rhs = Self> { type Output; fn add(self, rhs: Rhs) -> Self::Output; }
    pub trait Sub<Rhs = Self> { type Output; fn sub(self, rhs: Rhs) -> Self::Output; }
    pub trait Mul<Rhs = Self> { type Output; fn mul(self, rhs: Rhs) -> Self::Output; }
    pub trait Div<Rhs = Self> { type Output; fn div(self, rhs: Rhs) -> Self::Output; }
    pub trait Rem<Rhs = Self> { type Output; fn rem(self, rhs: Rhs) -> Self::Output; }
    pub trait BitAnd<Rhs = Self> { type Output; fn bitand(self, rhs: Rhs) -> Self::Output; }
    pub trait BitOr<Rhs = Self> { type Output; fn bitor(self, rhs: Rhs) -> Self::Output; }
    pub trait BitXor<Rhs = Self> { type Output; fn bitxor(self, rhs: Rhs) -> Self::Output; }
    pub trait Shl<Rhs = Self> { type Output; fn shl(self, rhs: Rhs) -> Self::Output; }
    pub trait Shr<Rhs = Self> { type Output; fn shr(self, rhs: Rhs) -> Self::Output; }
    pub trait Neg { type Output; fn neg(self) -> Self::Output; }
    pub trait Not { type Output; fn not(self) -> Self::Output; }
    pub trait AddAssign<Rhs = Self> { fn add_assign(&mut self, rhs: Rhs); }
    pub trait SubAssign<Rhs = Self> { fn sub_assign(&mut self, rhs: Rhs); }
    pub trait MulAssign<Rhs = Self> { fn mul_assign(&mut self, rhs: Rhs); }
    pub trait DivAssign<Rhs = Self> { fn div_assign(&mut self, rhs: Rhs); }
    pub trait RemAssign<Rhs = Self> { fn rem_assign(&mut self, rhs: Rhs); }
    pub trait BitAndAssign<Rhs = Self> { fn bitand_assign(&mut self, rhs: Rhs); }
    pub trait BitOrAssign<Rhs = Self> { fn bitor_assign(&mut self, rhs: Rhs); }
    pub trait BitXorAssign<Rhs = Self> { fn bitxor_assign(&mut self, rhs: Rhs); }
    pub trait ShlAssign<Rhs = Self> { fn shl_assign(&mut self, rhs: Rhs); }
    pub trait ShrAssign<Rhs = Self> { fn shr_assign(&mut self, rhs: Rhs); }
    pub trait Index<Idx> { type Output; fn index(&self, index: Idx) -> &Self::Output; }
    pub trait IndexMut<Idx>: Index<Idx> { fn index_mut(&mut self, index: Idx) -> &mut Self::Output; }
    pub trait Deref { type Target; fn deref(&self) -> &Self::Target; }
    pub trait DerefMut: Deref { fn deref_mut(&mut self) -> &mut Self::Target; }
    pub trait Drop { fn drop(&mut self); }

    #[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub struct Range<Idx> { pub start: Idx, pub end: Idx }
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RangeInclusive<Idx> { start: Idx, end: Idx, exhausted: bool }
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RangeFrom<Idx> { pub start: Idx }
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RangeTo<Idx> { pub end: Idx }
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RangeToInclusive<Idx> { pub end: Idx }
    #[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub struct RangeFull;

    impl<Idx> RangeInclusive<Idx> {
        pub fn new(start: Idx, end: Idx) -> RangeInclusive<Idx> { RangeInclusive { start: start, end: end, exhausted: false } }
        pub fn start(&self) -> &Idx { &self.start }
        pub fn end(&self) -> &Idx { &self.end }
        pub fn into_inner(self) -> (Idx, Idx) { (self.start, self.end) }
    }

    impl<Idx: PartialOrd> Range<Idx> {
        pub fn contains(&self, item: &Idx) -> bool { *item >= self.start && *item < self.end }
        pub fn is_empty(&self) -> bool { !(self.start < self.end) }
    }
    impl<Idx: PartialOrd> RangeInclusive<Idx> {
        pub fn contains(&self, item: &Idx) -> bool { *item >= self.start && *item <= self.end }
        pub fn is_empty(&self) -> bool { self.exhausted || !(self.start <= self.end) }
    }
    impl<Idx: PartialOrd> RangeFrom<Idx> {
        pub fn contains(&self, item: &Idx) -> bool { *item >= self.start }
    }
    impl<Idx: PartialOrd> RangeTo<Idx> {
        pub fn contains(&self, item: &Idx) -> bool { *item < self.end }
    }
    impl<Idx: PartialOrd> RangeToInclusive<Idx> {
        pub fn contains(&self, item: &Idx) -> bool { *item <= self.end }
    }

    impl<A: Copy> Iterator for Range<A> {
        type Item = A;
        fn next(&mut self) -> Option<A> { crate::intrinsics::range_next(self) }
    }
    impl<A: Copy> Iterator for RangeInclusive<A> {
        type Item = A;
        fn next(&mut self) -> Option<A> { crate::intrinsics::range_inclusive_next(self) }
    }
    impl<A: Copy> Iterator for RangeFrom<A> {
        type Item = A;
        fn next(&mut self) -> Option<A> { crate::intrinsics::range_from_next(self) }
    }

    impl<Idx: fmt::Debug> fmt::Debug for Range<Idx> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{:?}..{:?}", self.start, self.end) }
    }
    impl<Idx: fmt::Debug> fmt::Debug for RangeInclusive<Idx> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{:?}..={:?}", self.start, self.end) }
    }
    impl<Idx: fmt::Debug> fmt::Debug for RangeFrom<Idx> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{:?}..", self.start) }
    }
    impl<Idx: fmt::Debug> fmt::Debug for RangeTo<Idx> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "..{:?}", self.end) }
    }
    impl fmt::Debug for RangeFull {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "..") }
    }

    pub trait RangeBounds {}
    impl<T> RangeBounds for Range<T> {}
    impl<T> RangeBounds for RangeInclusive<T> {}
    impl<T> RangeBounds for RangeFrom<T> {}
    impl<T> RangeBounds for RangeTo<T> {}
    impl<T> RangeBounds for RangeToInclusive<T> {}
    impl RangeBounds for RangeFull {}
}

pub mod hash {
}

pub mod str {
    use crate::iter::Lazy;

    pub trait Pattern {}
    impl Pattern for char {}
    impl Pattern for &str {}
    impl Pattern for &&str {}
    impl Pattern for &String {}
    impl Pattern for &[char] {}
    impl<const N: usize> Pattern for [char; N] {}
    impl<const N: usize> Pattern for &[char; N] {}
    impl<F: FnMut(char) -> bool> Pattern for F {}

    pub trait FromStr: Sized {
        type Err;
        fn from_str(s: &str) -> Result<Self, Self::Err>;
    }

    impl str {
        pub fn char_indices(&self) -> Lazy<(usize, char)>;
        pub fn split<P: Pattern>(&self, pat: P) -> Lazy<&str>;
        pub fn rsplit<P: Pattern>(&self, pat: P) -> Lazy<&str>;
        pub fn splitn<P: Pattern>(&self, n: usize, pat: P) -> Lazy<&str>;
        pub fn rsplitn<P: Pattern>(&self, n: usize, pat: P) -> Lazy<&str>;
        pub fn split_terminator<P: Pattern>(&self, pat: P) -> Lazy<&str>;
        pub fn split_inclusive<P: Pattern>(&self, pat: P) -> Lazy<&str>;
        pub fn split_whitespace(&self) -> Lazy<&str>;
        pub fn split_ascii_whitespace(&self) -> Lazy<&str>;
        pub fn lines(&self) -> Lazy<&str>;
        pub fn split_once<P: Pattern>(&self, delimiter: P) -> Option<(&str, &str)>;
        pub fn rsplit_once<P: Pattern>(&self, delimiter: P) -> Option<(&str, &str)>;
        pub fn split_at(&self, mid: usize) -> (&str, &str);
        pub fn trim(&self) -> &str;
        pub fn trim_start(&self) -> &str;
        pub fn trim_end(&self) -> &str;
        pub fn trim_left(&self) -> &str;
        pub fn trim_right(&self) -> &str;
        pub fn trim_matches<P: Pattern>(&self, pat: P) -> &str;
        pub fn trim_start_matches<P: Pattern>(&self, pat: P) -> &str;
        pub fn trim_end_matches<P: Pattern>(&self, pat: P) -> &str;
        pub fn strip_prefix<P: Pattern>(&self, prefix: P) -> Option<&str>;
        pub fn strip_suffix<P: Pattern>(&self, suffix: P) -> Option<&str>;
        pub fn contains<P: Pattern>(&self, pat: P) -> bool;
        pub fn starts_with<P: Pattern>(&self, pat: P) -> bool;
        pub fn ends_with<P: Pattern>(&self, pat: P) -> bool;
        pub fn find<P: Pattern>(&self, pat: P) -> Option<usize>;
        pub fn rfind<P: Pattern>(&self, pat: P) -> Option<usize>;
        pub fn matches<P: Pattern>(&self, pat: P) -> Lazy<&str>;
        pub fn rmatches<P: Pattern>(&self, pat: P) -> Lazy<&str>;
        pub fn match_indices<P: Pattern>(&self, pat: P) -> Lazy<(usize, &str)>;
        pub fn replace<P: Pattern>(&self, from: P, to: &str) -> String;
        pub fn replacen<P: Pattern>(&self, from: P, to: &str, count: usize) -> String;
        pub fn to_uppercase(&self) -> String;
        pub fn to_lowercase(&self) -> String;
        pub fn to_ascii_uppercase(&self) -> String;
        pub fn to_ascii_lowercase(&self) -> String;
        pub fn repeat(&self, n: usize) -> String;
        pub fn get<R: crate::ops::RangeBounds>(&self, range: R) -> Option<&str>;
        pub fn is_char_boundary(&self, index: usize) -> bool;
        pub fn as_bytes(&self) -> &[u8];
        pub fn eq_ignore_ascii_case(&self, other: &str) -> bool;
        pub fn is_ascii(&self) -> bool;
        pub fn parse<F: FromStr>(&self) -> Result<F, F::Err> { F::from_str(self) }
        pub fn escape_default(&self) -> String;
        pub fn escape_debug(&self) -> String;
    }
}

pub mod string {
    use crate::iter::Lazy;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct FromUtf8Error { bytes: Vec<u8> }
    impl FromUtf8Error {
        pub fn into_bytes(self) -> Vec<u8> { self.bytes }
    }
    impl crate::fmt::Display for FromUtf8Error {
        fn fmt(&self, f: &mut crate::fmt::Formatter) -> crate::fmt::Result { write!(f, "invalid utf-8 sequence") }
    }

    pub trait ToString {
        fn to_string(&self) -> String;
    }

    impl String {
        pub fn with_capacity(capacity: usize) -> String { String::new() }
        pub fn capacity(&self) -> usize { self.len() }
        pub fn reserve(&mut self, additional: usize) {}
        pub fn shrink_to_fit(&mut self) {}
        pub fn insert(&mut self, idx: usize, ch: char);
        pub fn insert_str(&mut self, idx: usize, string: &str);
        pub fn remove(&mut self, idx: usize) -> char;
        pub fn retain<F: FnMut(char) -> bool>(&mut self, f: F);
        pub fn split_off(&mut self, at: usize) -> String;
        pub fn drain<R: crate::ops::RangeBounds>(&mut self, range: R) -> Lazy<char>;
        pub fn extend<I: IntoIterator>(&mut self, iter: I);
        pub fn into_bytes(self) -> Vec<u8>;
        pub fn from_utf8(vec: Vec<u8>) -> Result<String, FromUtf8Error>;
        pub fn from_utf8_lossy(v: &[u8]) -> String;
        pub fn replace_range<R: crate::ops::RangeBounds>(&mut self, range: R, replace_with: &str);
        pub fn into_boxed_str(self) -> String { self }
    }
}

pub mod char {
    impl char {
        pub fn is_alphabetic(self) -> bool;
        pub fn is_numeric(self) -> bool;
        pub fn is_alphanumeric(self) -> bool;
        pub fn is_whitespace(self) -> bool;
        pub fn is_uppercase(self) -> bool;
        pub fn is_lowercase(self) -> bool;
        pub fn is_control(self) -> bool;
        pub fn is_ascii(&self) -> bool;
        pub fn is_ascii_digit(&self) -> bool;
        pub fn is_ascii_hexdigit(&self) -> bool;
        pub fn is_ascii_alphabetic(&self) -> bool;
        pub fn is_ascii_alphanumeric(&self) -> bool;
        pub fn is_ascii_uppercase(&self) -> bool;
        pub fn is_ascii_lowercase(&self) -> bool;
        pub fn is_ascii_punctuation(&self) -> bool;
        pub fn is_ascii_whitespace(&self) -> bool;
        pub fn is_ascii_graphic(&self) -> bool;
        pub fn is_ascii_control(&self) -> bool;
        pub fn is_digit(self, radix: u32) -> bool;
        pub fn to_digit(self, radix: u32) -> Option<u32>;
        pub fn to_ascii_uppercase(&self) -> char;
        pub fn to_ascii_lowercase(&self) -> char;
        pub fn make_ascii_uppercase(&mut self);
        pub fn make_ascii_lowercase(&mut self);
        pub fn to_uppercase(self) -> crate::iter::Lazy<char>;
        pub fn to_lowercase(self) -> crate::iter::Lazy<char>;
        pub fn eq_ignore_ascii_case(&self, other: &char) -> bool;
        pub fn len_utf8(self) -> usize;
        pub fn from_u32(i: u32) -> Option<char>;
        pub fn from_digit(num: u32, radix: u32) -> Option<char>;
    }
}

pub mod f64 {
    pub mod consts {
        pub const PI: f64 = 3.14159265358979323846264338327950288_f64;
        pub const TAU: f64 = 6.28318530717958647692528676655900577_f64;
        pub const E: f64 = 2.71828182845904523536028747135266250_f64;
        pub const SQRT_2: f64 = 1.41421356237309504880168872420969808_f64;
        pub const FRAC_1_SQRT_2: f64 = 0.707106781186547524400844362104849039_f64;
        pub const LN_2: f64 = 0.693147180559945309417232121458176568_f64;
        pub const LN_10: f64 = 2.30258509299404568401799145468436421_f64;
        pub const LOG2_E: f64 = 1.44269504088896340735992468100189214_f64;
        pub const LOG10_E: f64 = 0.434294481903251827651128918916605082_f64;
        pub const FRAC_PI_2: f64 = 1.57079632679489661923132169163975144_f64;
        pub const FRAC_PI_3: f64 = 1.04719755119659774615421446109316763_f64;
        pub const FRAC_PI_4: f64 = 0.785398163397448309615660845819875721_f64;
        pub const FRAC_PI_6: f64 = 0.52359877559829887307710723054658381_f64;
        pub const FRAC_1_PI: f64 = 0.318309886183790671537767526745028724_f64;
        pub const FRAC_2_PI: f64 = 0.636619772367581343075535053490057448_f64;
    }
}

pub mod f32 {
    pub mod consts {
        pub const PI: f32 = 3.14159265358979323846264338327950288_f32;
        pub const TAU: f32 = 6.28318530717958647692528676655900577_f32;
        pub const E: f32 = 2.71828182845904523536028747135266250_f32;
        pub const SQRT_2: f32 = 1.41421356237309504880168872420969808_f32;
        pub const LN_2: f32 = 0.693147180559945309417232121458176568_f32;
        pub const FRAC_PI_2: f32 = 1.57079632679489661923132169163975144_f32;
    }
}

pub mod convert {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Infallible {}

    pub trait From<T>: Sized { fn from(value: T) -> Self; }
    pub trait Into<T>: Sized { fn into(self) -> T; }
    pub trait TryFrom<T>: Sized { type Error; fn try_from(value: T) -> Result<Self, Self::Error>; }
    pub trait TryInto<T>: Sized { type Error; fn try_into(self) -> Result<T, Self::Error>; }
    pub trait AsRef<T> { fn as_ref(&self) -> &T; }
    pub trait AsMut<T> { fn as_mut(&mut self) -> &mut T; }

    impl<T, U: From<T>> Into<U> for T {
        fn into(self) -> U { U::from(self) }
    }
    impl<T> From<T> for T {
        fn from(value: T) -> T { value }
    }
    impl<T, U: TryFrom<T>> TryInto<U> for T {
        type Error = U::Error;
        fn try_into(self) -> Result<U, U::Error> { U::try_from(self) }
    }

    impl From<&str> for String { fn from(value: &str) -> String { value.to_string() } }
    impl From<&String> for String { fn from(value: &String) -> String { value.clone() } }
    impl From<char> for String { fn from(value: char) -> String { value.to_string() } }
    impl From<&mut str> for String { fn from(value: &mut str) -> String { value.to_string() } }
    impl<T> From<T> for Option<T> { fn from(value: T) -> Option<T> { Some(value) } }
    impl<T> From<T> for Box<T> { fn from(value: T) -> Box<T> { Box::new(value) } }
    impl<T: Clone> From<&[T]> for Vec<T> { fn from(value: &[T]) -> Vec<T> { value.to_vec() } }
    impl<T: Clone> From<&Vec<T>> for Vec<T> { fn from(value: &Vec<T>) -> Vec<T> { value.clone() } }
    impl<T, const N: usize> From<[T; N]> for Vec<T> { fn from(value: [T; N]) -> Vec<T> { value.into_iter().collect() } }
    impl From<String> for Vec<u8> { fn from(value: String) -> Vec<u8> { value.into_bytes() } }
    impl From<&str> for Vec<u8> { fn from(value: &str) -> Vec<u8> { value.as_bytes().to_vec() } }
    impl From<u8> for char { fn from(value: u8) -> char { value as char } }
    impl From<char> for u32 { fn from(value: char) -> u32 { value as u32 } }
    impl From<char> for u64 { fn from(value: char) -> u64 { value as u64 } }
    impl From<bool> for i32 { fn from(value: bool) -> i32 { value as i32 } }
    impl From<bool> for u8 { fn from(value: bool) -> u8 { value as u8 } }
    impl From<bool> for usize { fn from(value: bool) -> usize { value as usize } }
    impl From<bool> for u32 { fn from(value: bool) -> u32 { value as u32 } }
    impl From<bool> for i64 { fn from(value: bool) -> i64 { value as i64 } }
    impl From<bool> for u64 { fn from(value: bool) -> u64 { value as u64 } }
    impl From<f32> for f64 { fn from(value: f32) -> f64 { value as f64 } }
    impl TryFrom<u32> for char {
        type Error = crate::num::TryFromIntError;
        fn try_from(value: u32) -> Result<char, crate::num::TryFromIntError> {
            match char::from_u32(value) { Some(c) => Ok(c), None => Err(crate::num::TryFromIntError) }
        }
    }

    impl AsRef<str> for str { fn as_ref(&self) -> &str { self } }
    impl AsRef<str> for String { fn as_ref(&self) -> &str { self.as_str() } }
    impl<T> AsRef<[T]> for [T] { fn as_ref(&self) -> &[T] { self } }
    impl<T> AsRef<[T]> for Vec<T> { fn as_ref(&self) -> &[T] { self.as_slice() } }
    impl AsRef<[u8]> for str { fn as_ref(&self) -> &[u8] { self.as_bytes() } }
    impl AsRef<[u8]> for String { fn as_ref(&self) -> &[u8] { self.as_bytes() } }
}

pub mod error {
    use crate::fmt::{Debug, Display};

    pub trait Error: Debug + Display {
        fn source(&self) -> Option<&dyn Error> { None }
        fn description(&self) -> &str { "description() is deprecated; use Display" }
    }

    impl<E: Error> From<E> for Box<dyn Error> {
        fn from(err: E) -> Box<dyn Error> { Box::new(err) }
    }
    impl From<&str> for Box<dyn Error> {
        fn from(err: &str) -> Box<dyn Error> { Box::new(crate::error::StringError(err.to_string())) }
    }
    impl From<String> for Box<dyn Error> {
        fn from(err: String) -> Box<dyn Error> { Box::new(crate::error::StringError(err)) }
    }

    pub struct StringError(String);
    impl Debug for StringError {
        fn fmt(&self, f: &mut crate::fmt::Formatter) -> crate::fmt::Result { write!(f, "{:?}", self.0) }
    }
    impl Display for StringError {
        fn fmt(&self, f: &mut crate::fmt::Formatter) -> crate::fmt::Result { write!(f, "{}", self.0) }
    }
    impl Error for StringError {}

    impl Error for crate::num::ParseIntError {}
    impl Error for crate::num::ParseFloatError {}
    impl Error for crate::num::ParseBoolError {}
    impl Error for crate::num::ParseCharError {}
    impl Error for crate::num::TryFromIntError {}
    impl Error for crate::fmt::Error {}
    impl Error for crate::string::FromUtf8Error {}
}

pub mod collections {
    use crate::iter::Lazy;
    use crate::ops::{Index, IndexMut, RangeBounds};
    use crate::fmt;

    pub mod hash_map {
        pub use super::HashMap;
        pub use super::Entry;
    }
    pub mod btree_map {
        pub use super::BTreeMap;
        pub use super::Entry;
    }
    pub mod hash_set {
        pub use super::HashSet;
    }
    pub mod btree_set {
        pub use super::BTreeSet;
    }
    pub mod vec_deque {
        pub use super::VecDeque;
    }
    pub mod binary_heap {
        pub use super::BinaryHeap;
    }

    // Maps and sets hold a Python dict at runtime; these fields only give them a Rust type.
    #[derive(Clone, PartialEq, Eq, Default)]
    pub struct HashMap<K, V> { entries: Vec<(K, V)> }
    #[derive(Clone, PartialEq, Eq, Default, PartialOrd, Ord)]
    pub struct BTreeMap<K, V> { entries: Vec<(K, V)> }
    #[derive(Clone, PartialEq, Eq, Default)]
    pub struct HashSet<T> { entries: Vec<T> }
    #[derive(Clone, PartialEq, Eq, Default, PartialOrd, Ord)]
    pub struct BTreeSet<T> { entries: Vec<T> }

    pub struct Entry<'a, M, K, V> { map: &'a mut M, key: K, value: Option<V> }

    impl<'a, M, K, V> Entry<'a, M, K, V> {
        pub fn or_insert(self, default: V) -> &'a mut V { crate::intrinsics::map_or_insert(self.map, self.key, default) }
        pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
            if crate::intrinsics::map_has(&*self.map, &self.key) {
                crate::intrinsics::map_slot(self.map, &self.key)
            } else {
                crate::intrinsics::map_or_insert(self.map, self.key, default())
            }
        }
        pub fn or_insert_with_key<F: FnOnce(&K) -> V>(self, default: F) -> &'a mut V {
            if crate::intrinsics::map_has(&*self.map, &self.key) {
                crate::intrinsics::map_slot(self.map, &self.key)
            } else {
                let value = default(&self.key);
                crate::intrinsics::map_or_insert(self.map, self.key, value)
            }
        }
        pub fn or_default(self) -> &'a mut V where V: Default {
            if crate::intrinsics::map_has(&*self.map, &self.key) {
                crate::intrinsics::map_slot(self.map, &self.key)
            } else {
                crate::intrinsics::map_or_insert(self.map, self.key, V::default())
            }
        }
        pub fn and_modify<F: FnOnce(&mut V)>(self, f: F) -> Entry<'a, M, K, V> {
            if crate::intrinsics::map_has(&*self.map, &self.key) {
                f(crate::intrinsics::map_slot(self.map, &self.key));
            }
            self
        }
        pub fn key(&self) -> &K { &self.key }
    }

    macro_rules! map_methods {
        ($map:ident) => {
            impl<K, V> $map<K, V> {
                pub fn new() -> $map<K, V> { crate::intrinsics::map_new() }
                pub fn with_capacity(capacity: usize) -> $map<K, V> { crate::intrinsics::map_new() }
                pub fn len(&self) -> usize { crate::intrinsics::map_len(self) }
                pub fn is_empty(&self) -> bool { crate::intrinsics::map_len(self) == 0 }
                pub fn clear(&mut self) { crate::intrinsics::map_clear(self) }
                pub fn insert(&mut self, key: K, value: V) -> Option<V> { crate::intrinsics::map_insert(self, key, value) }
                pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V> { crate::intrinsics::map_get(self, key) }
                pub fn get_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut V> { crate::intrinsics::map_get(self, key) }
                pub fn get_key_value<Q: ?Sized>(&self, key: &Q) -> Option<(&K, &V)> { crate::intrinsics::map_get_key_value(self, key) }
                pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool { crate::intrinsics::map_has(self, key) }
                pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<V> { crate::intrinsics::map_remove(self, key) }
                pub fn remove_entry<Q: ?Sized>(&mut self, key: &Q) -> Option<(K, V)> { crate::intrinsics::map_remove_entry(self, key) }
                pub fn entry(&mut self, key: K) -> Entry<'_, $map<K, V>, K, V> { Entry { map: self, key: key, value: None } }
                pub fn keys(&self) -> Lazy<&K> { crate::intrinsics::map_keys(self) }
                pub fn values(&self) -> Lazy<&V> { crate::intrinsics::map_values(self) }
                pub fn values_mut(&mut self) -> Lazy<&mut V> { crate::intrinsics::map_values(self) }
                pub fn into_keys(self) -> Lazy<K> { crate::intrinsics::map_keys(self) }
                pub fn into_values(self) -> Lazy<V> { crate::intrinsics::map_values(self) }
                pub fn iter(&self) -> Lazy<(&K, &V)> { crate::intrinsics::map_items(self) }
                pub fn iter_mut(&mut self) -> Lazy<(&K, &mut V)> { crate::intrinsics::map_items(self) }
                pub fn drain(&mut self) -> Lazy<(K, V)> { crate::intrinsics::map_drain(self) }
                pub fn retain<F: FnMut(&K, &mut V) -> bool>(&mut self, f: F) { crate::intrinsics::map_retain(self, f) }
                pub fn extend<I: IntoIterator>(&mut self, iter: I) { crate::intrinsics::map_extend(self, iter) }
                pub fn first_key_value(&self) -> Option<(&K, &V)> { crate::intrinsics::map_end(self, false) }
                pub fn last_key_value(&self) -> Option<(&K, &V)> { crate::intrinsics::map_end(self, true) }
                pub fn pop_first(&mut self) -> Option<(K, V)> { crate::intrinsics::map_pop(self, false) }
                pub fn pop_last(&mut self) -> Option<(K, V)> { crate::intrinsics::map_pop(self, true) }
                pub fn range<R: RangeBounds>(&self, range: R) -> Lazy<(&K, &V)> { crate::intrinsics::map_range(self, range) }
            }
            impl<K, V> IntoIterator for $map<K, V> {
                type IntoIter = Lazy<(K, V)>;
                fn into_iter(self) -> Lazy<(K, V)> { crate::intrinsics::map_items(self) }
            }
            impl<K, V> IntoIterator for &$map<K, V> {
                type IntoIter = Lazy<(&K, &V)>;
                fn into_iter(self) -> Lazy<(&K, &V)> { crate::intrinsics::map_items(self) }
            }
            impl<K, V> IntoIterator for &mut $map<K, V> {
                type IntoIter = Lazy<(&K, &mut V)>;
                fn into_iter(self) -> Lazy<(&K, &mut V)> { crate::intrinsics::map_items(self) }
            }
            impl<K, Q: ?Sized, V> Index<&Q> for $map<K, V> {
                type Output = V;
                fn index(&self, key: &Q) -> &V {
                    match crate::intrinsics::map_get(self, key) { Some(value) => value, None => panic!("key not found") }
                }
            }
            impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for $map<K, V> {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { crate::intrinsics::map_debug(self, f) }
            }
        };
    }
    map_methods!(HashMap);
    map_methods!(BTreeMap);

    macro_rules! set_methods {
        ($set:ident) => {
            impl<T> $set<T> {
                pub fn new() -> $set<T> { crate::intrinsics::map_new() }
                pub fn with_capacity(capacity: usize) -> $set<T> { crate::intrinsics::map_new() }
                pub fn len(&self) -> usize { crate::intrinsics::map_len(self) }
                pub fn is_empty(&self) -> bool { crate::intrinsics::map_len(self) == 0 }
                pub fn clear(&mut self) { crate::intrinsics::map_clear(self) }
                pub fn insert(&mut self, value: T) -> bool { crate::intrinsics::set_insert(self, value) }
                pub fn contains<Q: ?Sized>(&self, value: &Q) -> bool { crate::intrinsics::map_has(self, value) }
                pub fn get<Q: ?Sized>(&self, value: &Q) -> Option<&T> { crate::intrinsics::set_get(self, value) }
                pub fn remove<Q: ?Sized>(&mut self, value: &Q) -> bool { crate::intrinsics::set_remove(self, value) }
                pub fn take<Q: ?Sized>(&mut self, value: &Q) -> Option<T> { crate::intrinsics::set_take(self, value) }
                pub fn iter(&self) -> Lazy<&T> { crate::intrinsics::map_keys(self) }
                pub fn drain(&mut self) -> Lazy<T> { crate::intrinsics::set_drain(self) }
                pub fn retain<F: FnMut(&T) -> bool>(&mut self, f: F) { crate::intrinsics::set_retain(self, f) }
                pub fn extend<I: IntoIterator>(&mut self, iter: I) { crate::intrinsics::set_extend(self, iter) }
                pub fn union<'a>(&'a self, other: &'a $set<T>) -> Lazy<&'a T> { crate::intrinsics::set_combine(self, other, 0) }
                pub fn intersection<'a>(&'a self, other: &'a $set<T>) -> Lazy<&'a T> { crate::intrinsics::set_combine(self, other, 1) }
                pub fn difference<'a>(&'a self, other: &'a $set<T>) -> Lazy<&'a T> { crate::intrinsics::set_combine(self, other, 2) }
                pub fn symmetric_difference<'a>(&'a self, other: &'a $set<T>) -> Lazy<&'a T> { crate::intrinsics::set_combine(self, other, 3) }
                pub fn is_subset(&self, other: &$set<T>) -> bool { crate::intrinsics::set_relation(self, other, 0) }
                pub fn is_superset(&self, other: &$set<T>) -> bool { crate::intrinsics::set_relation(other, self, 0) }
                pub fn is_disjoint(&self, other: &$set<T>) -> bool { crate::intrinsics::set_relation(self, other, 1) }
                pub fn first(&self) -> Option<&T> { crate::intrinsics::set_end(self, false) }
                pub fn last(&self) -> Option<&T> { crate::intrinsics::set_end(self, true) }
                pub fn pop_first(&mut self) -> Option<T> { crate::intrinsics::set_pop(self, false) }
                pub fn pop_last(&mut self) -> Option<T> { crate::intrinsics::set_pop(self, true) }
                pub fn range<R: RangeBounds>(&self, range: R) -> Lazy<&T> { crate::intrinsics::set_range(self, range) }
            }
            impl<T> IntoIterator for $set<T> {
                type IntoIter = Lazy<T>;
                fn into_iter(self) -> Lazy<T> { crate::intrinsics::map_keys(self) }
            }
            impl<T> IntoIterator for &$set<T> {
                type IntoIter = Lazy<&T>;
                fn into_iter(self) -> Lazy<&T> { crate::intrinsics::map_keys(self) }
            }
            impl<T: fmt::Debug> fmt::Debug for $set<T> {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { crate::intrinsics::map_debug(self, f) }
            }
        };
    }
    set_methods!(HashSet);
    set_methods!(BTreeSet);

    #[derive(Clone, PartialEq, Eq, Default, PartialOrd, Ord, Hash)]
    pub struct VecDeque<T> { items: Vec<T> }

    impl<T> VecDeque<T> {
        pub fn new() -> VecDeque<T> { VecDeque { items: Vec::new() } }
        pub fn with_capacity(capacity: usize) -> VecDeque<T> { VecDeque { items: Vec::new() } }
        pub fn len(&self) -> usize { self.items.len() }
        pub fn is_empty(&self) -> bool { self.items.is_empty() }
        pub fn clear(&mut self) { self.items.clear() }
        pub fn push_back(&mut self, value: T) { self.items.push(value) }
        pub fn push_front(&mut self, value: T) { self.items.insert(0, value) }
        pub fn pop_back(&mut self) -> Option<T> { self.items.pop() }
        pub fn pop_front(&mut self) -> Option<T> { if self.items.is_empty() { None } else { Some(self.items.remove(0)) } }
        pub fn front(&self) -> Option<&T> { self.items.first() }
        pub fn back(&self) -> Option<&T> { self.items.last() }
        pub fn front_mut(&mut self) -> Option<&mut T> { self.items.first_mut() }
        pub fn back_mut(&mut self) -> Option<&mut T> { self.items.last_mut() }
        pub fn get(&self, index: usize) -> Option<&T> { self.items.get(index) }
        pub fn get_mut(&mut self, index: usize) -> Option<&mut T> { self.items.get_mut(index) }
        pub fn insert(&mut self, index: usize, value: T) { self.items.insert(index, value) }
        pub fn remove(&mut self, index: usize) -> Option<T> { if index < self.items.len() { Some(self.items.remove(index)) } else { None } }
        pub fn contains(&self, x: &T) -> bool where T: PartialEq { self.items.contains(x) }
        pub fn truncate(&mut self, len: usize) { self.items.truncate(len) }
        pub fn retain<F: FnMut(&T) -> bool>(&mut self, f: F) { self.items.retain(f) }
        pub fn swap(&mut self, i: usize, j: usize) { self.items.swap(i, j) }
        pub fn rotate_left(&mut self, n: usize) { self.items.rotate_left(n) }
        pub fn rotate_right(&mut self, n: usize) { self.items.rotate_right(n) }
        pub fn make_contiguous(&mut self) -> &mut [T] { &mut self.items[..] }
        pub fn extend<I: IntoIterator>(&mut self, iter: I) { self.items.extend(iter) }
        pub fn iter(&self) -> Lazy<&T> { crate::intrinsics::lazy_elements(&self.items) }
        pub fn iter_mut(&mut self) -> Lazy<&mut T> { crate::intrinsics::lazy_elements(&mut self.items) }
        pub fn drain<R: RangeBounds>(&mut self, range: R) -> Lazy<T> { self.items.drain(range) }
        pub fn append(&mut self, other: &mut VecDeque<T>) { self.items.append(&mut other.items) }
        pub fn split_off(&mut self, at: usize) -> VecDeque<T> { VecDeque { items: self.items.split_off(at) } }
        pub fn resize(&mut self, len: usize, value: T) where T: Clone { self.items.resize(len, value) }
        pub fn as_slices(&self) -> (&[T], &[T]) { (&self.items[..], &self.items[0..0]) }
        pub fn range<R: RangeBounds>(&self, range: R) -> Lazy<&T> { crate::intrinsics::lazy_range(&self.items, range) }
    }
    impl<T> Index<usize> for VecDeque<T> {
        type Output = T;
        fn index(&self, index: usize) -> &T { &self.items[index] }
    }
    impl<T> IndexMut<usize> for VecDeque<T> {
        fn index_mut(&mut self, index: usize) -> &mut T { &mut self.items[index] }
    }
    impl<T> IntoIterator for VecDeque<T> {
        type IntoIter = Lazy<T>;
        fn into_iter(self) -> Lazy<T> { crate::intrinsics::lazy_elements(self.items) }
    }
    impl<T> IntoIterator for &VecDeque<T> {
        type IntoIter = Lazy<&T>;
        fn into_iter(self) -> Lazy<&T> { crate::intrinsics::lazy_elements(&self.items) }
    }
    impl<T: fmt::Debug> fmt::Debug for VecDeque<T> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{:?}", self.items) }
    }
    impl<T> From<Vec<T>> for VecDeque<T> {
        fn from(items: Vec<T>) -> VecDeque<T> { VecDeque { items: items } }
    }
    impl<T, const N: usize> From<[T; N]> for VecDeque<T> {
        fn from(items: [T; N]) -> VecDeque<T> { VecDeque { items: Vec::from(items) } }
    }
    impl<T> From<VecDeque<T>> for Vec<T> {
        fn from(deque: VecDeque<T>) -> Vec<T> { deque.items }
    }

    #[derive(Clone, Default)]
    pub struct BinaryHeap<T> { items: Vec<T> }

    impl<T: Ord> BinaryHeap<T> {
        pub fn new() -> BinaryHeap<T> { BinaryHeap { items: Vec::new() } }
        pub fn with_capacity(capacity: usize) -> BinaryHeap<T> { BinaryHeap { items: Vec::new() } }
        pub fn len(&self) -> usize { self.items.len() }
        pub fn is_empty(&self) -> bool { self.items.is_empty() }
        pub fn clear(&mut self) { self.items.clear() }
        pub fn peek(&self) -> Option<&T> { self.items.first() }
        pub fn push(&mut self, item: T) {
            self.items.push(item);
            let mut child = self.items.len() - 1;
            while child > 0 {
                let parent = (child - 1) / 2;
                if self.items[child] <= self.items[parent] { break; }
                self.items.swap(child, parent);
                child = parent;
            }
        }
        pub fn pop(&mut self) -> Option<T> {
            let last = self.items.pop()?;
            if self.items.is_empty() { return Some(last); }
            let top = crate::mem::replace(&mut self.items[0], last);
            let len = self.items.len();
            let mut parent = 0;
            loop {
                let left = 2 * parent + 1;
                if left >= len { break; }
                let right = left + 1;
                let mut largest = left;
                if right < len && self.items[right] > self.items[left] { largest = right; }
                if self.items[largest] <= self.items[parent] { break; }
                self.items.swap(parent, largest);
                parent = largest;
            }
            Some(top)
        }
        pub fn into_vec(self) -> Vec<T> { self.items }
        pub fn into_sorted_vec(self) -> Vec<T> { let mut items = self.items; items.sort(); items }
        pub fn iter(&self) -> Lazy<&T> { crate::intrinsics::lazy_elements(&self.items) }
        pub fn drain(&mut self) -> Lazy<T> { self.items.drain(..) }
        pub fn extend<It: Iterator<Item = T>, I: IntoIterator<IntoIter = It>>(&mut self, iter: I) {
            let mut items = iter.into_iter();
            while let Some(item) = items.next() { self.push(item); }
        }
        pub fn from_vec(items: Vec<T>) -> BinaryHeap<T> {
            let mut heap = BinaryHeap::new();
            for item in items { heap.push(item); }
            heap
        }
    }
    impl<T: Ord> From<Vec<T>> for BinaryHeap<T> {
        fn from(items: Vec<T>) -> BinaryHeap<T> { BinaryHeap::from_vec(items) }
    }
    impl<T> IntoIterator for BinaryHeap<T> {
        type IntoIter = Lazy<T>;
        fn into_iter(self) -> Lazy<T> { crate::intrinsics::lazy_elements(self.items) }
    }
    impl<T: fmt::Debug> fmt::Debug for BinaryHeap<T> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{:?}", self.items) }
    }
}
