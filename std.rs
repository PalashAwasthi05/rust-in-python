// The parts of Rust's standard library this interpreter provides, written in Rust.
// Body-less functions are intrinsics implemented in rust_in_python.py.

pub mod prelude {
    pub use crate::iter::Iterator;
    pub use crate::iter::IntoIterator;
    pub use crate::iter::Extend;
    pub use crate::iter::{DoubleEndedIterator, ExactSizeIterator, FromIterator};
    pub use crate::cmp::{Ordering, PartialEq, Eq, PartialOrd, Ord};
    pub use crate::mem::drop;
    pub use crate::convert::{From, Into, TryFrom, TryInto, AsRef, AsMut};
    pub use crate::ops::{Drop, Fn, FnMut, FnOnce, Deref, DerefMut};
    pub use crate::clone::Clone;
    pub use crate::default::Default;
    pub use crate::marker::Copy;
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
    impl<T, const N: usize> Join for [T; N] { type Output = Vec<T>; }
    impl<T> Join for &[T] { type Output = Vec<T>; }

    pub fn iter_max<I: crate::iter::Iterator>(iter: I, last: bool) -> Option<I::Item>;
    pub fn iter_collect<I: crate::iter::Iterator>(iter: I) -> Vec<I::Item>;
    pub fn iter_partition<I, F, R>(iter: I, f: F) -> R;
    pub fn iter_unzip<I, R>(iter: I) -> R;
    pub fn iter_try_fold<I, B, F, R>(iter: &mut I, init: B, f: F) -> R;
    pub fn iter_try_for_each<I, F, R>(iter: &mut I, f: F) -> R;
    pub fn iter_compare<I, J>(iter: I, other: J, ordering: i32) -> i32;
    pub fn iter_is_sorted<I>(iter: I) -> bool;
    pub fn iter_collect_into<I, R>(iter: I) -> R;
    pub fn iter_cycle<I, R>(iter: I) -> R;
    pub fn iter_chunks<I, R>(iter: I, size: usize) -> R;
    pub fn range_next<A>(range: &mut crate::ops::Range<A>) -> Option<A>;
    pub fn range_length<A>(range: &crate::ops::Range<A>) -> usize;
    pub fn wrapping_op<T>(left: T, right: T, operator: u8) -> T;
    pub fn wrapping_shift<T>(value: T, amount: usize, left: bool) -> T;
    pub fn range_inclusive_next<A>(range: &mut crate::ops::RangeInclusive<A>) -> Option<A>;
    pub fn range_from_next<A>(range: &mut crate::ops::RangeFrom<A>) -> Option<A>;
    pub fn range_nth<A>(range: &mut crate::ops::Range<A>, n: usize) -> Option<A>;
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
    pub fn duration_debug(nanos: u128, f: &mut crate::fmt::Formatter) -> crate::fmt::Result;
    pub fn debug_entry(value: &dyn crate::fmt::Debug, f: &crate::fmt::Formatter) -> String;
    pub fn arguments_text<A>(value: A) -> String;
    pub fn kind_fmt<T>(value: &T, f: &mut crate::fmt::Formatter, kind: char) -> crate::fmt::Result;
    pub fn hash_value<T>(value: &T) -> u64;
    pub fn track_drops();
    pub fn box_unsized<T, U: ?Sized>(value: T) -> Box<U>;
    pub fn pad_text(text: &str, width: Option<usize>, precision: Option<usize>, fill: char, align: u8) -> String;
    pub fn clock_nanos() -> u128;
    pub fn wall_nanos() -> u128;
    pub fn emit(text: &str, error: bool);
    pub fn stdin_read_line(buf: &mut String) -> usize;
    pub fn stdin_read_all(buf: &mut String) -> usize;
    pub fn stdin_lines() -> crate::iter::Lazy<crate::io::Result<String>>;
    pub fn env_args() -> crate::iter::Lazy<String>;
    pub fn env_var(key: &str) -> Option<String>;
    pub fn rc_deref<T: ?Sized>(rc: &crate::rc::Rc<T>) -> &T;
    pub fn refcell_value<'a, T>(cell: &'a crate::cell::RefCell<T>) -> &'a mut T;
    pub fn cell_value<'a, T>(cell: &'a crate::cell::Cell<T>) -> &'a mut T;
    pub fn lazy_range<C, B, R>(items: C, range: B) -> R;
    pub fn parse_float<T>(text: &str) -> Result<T, crate::num::ParseFloatError>;
    pub fn lazy_next<T>(iter: &mut crate::iter::Lazy<T>) -> Option<T>;
    pub fn extend_builtin<C: ?Sized, I>(collection: &mut C, iter: I);
    pub fn iter_size_hint<I: ?Sized>(iter: &I) -> (usize, Option<usize>);
    pub fn double_ended<I: ?Sized>(iter: &I) -> bool;
    pub fn next_back<I: crate::iter::Iterator + ?Sized>(iter: &mut I) -> Option<I::Item>;
    pub fn range_count<R>(range: &R) -> usize;
    pub fn range_next_back<A>(range: &mut crate::ops::Range<A>) -> Option<A>;
    pub fn range_inclusive_next_back<A>(range: &mut crate::ops::RangeInclusive<A>) -> Option<A>;
    pub fn lazy_next_back<T>(iter: &mut crate::iter::Lazy<T>) -> Option<T>;
    pub fn vec_to_array<T, const N: usize>(items: Vec<T>) -> [T; N];
    pub fn range_resolve<B>(range: B, len: usize) -> (usize, usize);
}

pub mod iter {
    pub trait Iterator {
        type Item;
        fn next(&mut self) -> Option<Self::Item>;

        fn size_hint(&self) -> (usize, Option<usize>) { crate::intrinsics::iter_size_hint(self) }
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
        fn chain<U>(self, other: U) -> Chain<Self, U::IntoIter> where Self: Sized, U: IntoIterator<Item = Self::Item> {
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
        fn rposition<P>(self, predicate: P) -> Option<usize> where Self: Sized, P: FnMut(Self::Item) -> bool {
            let mut items = crate::intrinsics::iter_collect(self);
            let mut predicate = predicate;
            let mut index = items.len();
            while let Some(item) = items.pop() {
                index -= 1;
                if predicate(item) { return Some(index); }
            }
            None
        }
        fn rev(self) -> Rev<Self> where Self: Sized {
            Rev { iter: self, buffer: None }
        }
        fn partition<B, F>(self, f: F) -> (B, B) where Self: Sized, B: Default + Extend<Self::Item>, F: FnMut(&Self::Item) -> bool {
            crate::intrinsics::iter_partition(self, f)
        }
        fn unzip<A, B, FromA, FromB>(self) -> (FromA, FromB) where Self: Sized, FromA: Default + Extend<A>, FromB: Default + Extend<B> {
            crate::intrinsics::iter_unzip(self)
        }
        fn try_fold<B, F, R>(&mut self, init: B, f: F) -> R where Self: Sized, F: FnMut(B, Self::Item) -> R {
            crate::intrinsics::iter_try_fold(self, init, f)
        }
        fn try_for_each<F, R>(&mut self, f: F) -> R where Self: Sized, F: FnMut(Self::Item) -> R {
            crate::intrinsics::iter_try_for_each(self, f)
        }
        fn eq<I: IntoIterator>(self, other: I) -> bool where Self: Sized { crate::intrinsics::iter_compare(self, other, 0) == 0 }
        fn ne<I: IntoIterator>(self, other: I) -> bool where Self: Sized { crate::intrinsics::iter_compare(self, other, 0) != 0 }
        fn lt<I: IntoIterator>(self, other: I) -> bool where Self: Sized { crate::intrinsics::iter_compare(self, other, 1) < 0 }
        fn le<I: IntoIterator>(self, other: I) -> bool where Self: Sized { crate::intrinsics::iter_compare(self, other, 1) <= 0 }
        fn gt<I: IntoIterator>(self, other: I) -> bool where Self: Sized { crate::intrinsics::iter_compare(self, other, 1) > 0 }
        fn ge<I: IntoIterator>(self, other: I) -> bool where Self: Sized { crate::intrinsics::iter_compare(self, other, 1) >= 0 }
        fn cmp<I: IntoIterator>(self, other: I) -> crate::cmp::Ordering where Self: Sized {
            let order = crate::intrinsics::iter_compare(self, other, 1);
            if order < 0 { crate::cmp::Ordering::Less } else if order > 0 { crate::cmp::Ordering::Greater } else { crate::cmp::Ordering::Equal }
        }
        fn is_sorted(self) -> bool where Self: Sized { crate::intrinsics::iter_is_sorted(self) }
        fn sum_by<B>(self) -> B where Self: Sized { crate::intrinsics::iter_collect_into(self) }
        fn cycle(self) -> Lazy<Self::Item> where Self: Sized { crate::intrinsics::iter_cycle(self) }
        fn array_chunks_vec(self, size: usize) -> Lazy<Vec<Self::Item>> where Self: Sized { crate::intrinsics::iter_chunks(self, size) }
    }

    // Built-in collections extend through their own inherent methods; these impls let inference
    // find a collection's element type, and user types implement extend themselves.
    pub trait Extend<A> {
        fn extend<T: IntoIterator<Item = A>>(&mut self, iter: T) { crate::intrinsics::extend_builtin(self, iter) }
    }
    impl<T> Extend<T> for Vec<T> {}
    impl<'a, T: Copy> Extend<&'a T> for Vec<T> {}
    impl Extend<char> for String {}
    impl<'a> Extend<&'a str> for String {}
    impl Extend<String> for String {}
    impl<T> Extend<T> for crate::collections::VecDeque<T> {}
    impl<T> Extend<T> for crate::collections::HashSet<T> {}
    impl<T> Extend<T> for crate::collections::BTreeSet<T> {}
    impl<K, V> Extend<(K, V)> for crate::collections::HashMap<K, V> {}
    impl<K, V> Extend<(K, V)> for crate::collections::BTreeMap<K, V> {}

    pub trait IntoIterator {
        type Item;
        type IntoIter: Iterator;
        fn into_iter(self) -> Self::IntoIter;
    }

    pub struct StepBy<I> { iter: I, step: usize, first: bool }
    impl<I: Iterator> Iterator for StepBy<I> {
        type Item = I::Item;
        fn size_hint(&self) -> (usize, Option<usize>) {
            let (low, high) = self.iter.size_hint();
            let step = self.step + 1;
            if self.first {
                let f = |n: usize| if n == 0 { 0 } else { 1 + (n - 1) / step };
                (f(low), high.map(f))
            } else {
                (low / step, high.map(|n| n / step))
            }
        }
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
        fn size_hint(&self) -> (usize, Option<usize>) {
            let (low, high) = self.second.size_hint();
            match &self.first {
                Some(first) => {
                    let (first_low, first_high) = first.size_hint();
                    let total = match (first_high, high) {
                        (Some(a), Some(b)) => a.checked_add(b),
                        _ => None,
                    };
                    (first_low.saturating_add(low), total)
                }
                None => (low, high),
            }
        }
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
        fn size_hint(&self) -> (usize, Option<usize>) {
            let (a_low, a_high) = self.a.size_hint();
            let (b_low, b_high) = self.b.size_hint();
            let high = match (a_high, b_high) {
                (Some(x), Some(y)) => Some(if x < y { x } else { y }),
                (Some(x), None) => Some(x),
                (None, Some(y)) => Some(y),
                (None, None) => None,
            };
            (if a_low < b_low { a_low } else { b_low }, high)
        }
        fn next(&mut self) -> Option<(A::Item, B::Item)> {
            let left = self.a.next()?;
            let right = self.b.next()?;
            Some((left, right))
        }
    }

    pub struct Map<I, F> { iter: I, f: F }
    impl<B, I: Iterator, F: FnMut(I::Item) -> B> Iterator for Map<I, F> {
        type Item = B;
        fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
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
        fn size_hint(&self) -> (usize, Option<usize>) { (0, self.iter.size_hint().1) }
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
        fn size_hint(&self) -> (usize, Option<usize>) { (0, self.iter.size_hint().1) }
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
        fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
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
        fn size_hint(&self) -> (usize, Option<usize>) {
            let extra = match &self.peeked {
                Some(None) => return (0, Some(0)),
                Some(Some(_)) => 1,
                None => 0,
            };
            let (low, high) = self.iter.size_hint();
            (low.saturating_add(extra), match high { Some(n) => n.checked_add(extra), None => None })
        }
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
        pub fn next_if_eq(&mut self, expected: &I::Item) -> Option<I::Item> where I::Item: PartialEq {
            let item = self.next()?;
            if item == *expected { return Some(item); }
            self.peeked = Some(Some(item));
            None
        }
        pub fn peek_mut(&mut self) -> Option<&mut I::Item> {
            if self.peeked.is_none() {
                self.peeked = Some(self.iter.next());
            }
            match &mut self.peeked {
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
        fn size_hint(&self) -> (usize, Option<usize>) { (0, self.iter.size_hint().1) }
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
        fn size_hint(&self) -> (usize, Option<usize>) {
            if self.done { (0, Some(0)) } else { (0, self.iter.size_hint().1) }
        }
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
        fn size_hint(&self) -> (usize, Option<usize>) {
            if self.done { (0, Some(0)) } else { (0, self.iter.size_hint().1) }
        }
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
        fn size_hint(&self) -> (usize, Option<usize>) {
            let (low, high) = self.iter.size_hint();
            (low.saturating_sub(self.n), high.map(|x| x.saturating_sub(self.n)))
        }
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
        fn size_hint(&self) -> (usize, Option<usize>) {
            if self.n == 0 { return (0, Some(0)); }
            let (low, high) = self.iter.size_hint();
            let low = if low < self.n { low } else { self.n };
            let high = match high {
                Some(x) if x < self.n => Some(x),
                _ => Some(self.n),
            };
            (low, high)
        }
        fn next(&mut self) -> Option<I::Item> {
            if self.n == 0 { return None; }
            self.n -= 1;
            self.iter.next()
        }
    }

    pub struct Scan<I, St, F> { iter: I, f: F, state: St }
    impl<B, I: Iterator, St, F: FnMut(&mut St, I::Item) -> Option<B>> Iterator for Scan<I, St, F> {
        type Item = B;
        fn size_hint(&self) -> (usize, Option<usize>) { (0, self.iter.size_hint().1) }
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
    impl<I: Iterator + ?Sized> Iterator for &mut I {
        type Item = I::Item;
        fn size_hint(&self) -> (usize, Option<usize>) { (**self).size_hint() }
        fn next(&mut self) -> Option<I::Item> { (**self).next() }
    }
    impl<T> Iterator for Lazy<T> {
        type Item = T;
        fn next(&mut self) -> Option<T> { crate::intrinsics::lazy_next(self) }
    }
    impl<T> Lazy<T> {
        pub fn next_back(&mut self) -> Option<T> { crate::intrinsics::lazy_next_back(self) }
        pub fn nth_back(&mut self, n: usize) -> Option<T> {
            for _ in 0..n {
                self.next_back()?;
            }
            self.next_back()
        }
    }

    pub struct Fuse<I> { iter: Option<I> }
    impl<I: Iterator> Iterator for Fuse<I> {
        type Item = I::Item;
        fn size_hint(&self) -> (usize, Option<usize>) {
            match &self.iter {
                Some(iter) => iter.size_hint(),
                None => (0, Some(0)),
            }
        }
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
        fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
        fn next(&mut self) -> Option<I::Item> {
            let item = self.iter.next()?;
            (self.f)(&item);
            Some(item)
        }
    }

    // Rev walks a DoubleEndedIterator from the back; other iterators are buffered on first use.
    pub struct Rev<I: Iterator> { iter: I, buffer: Option<Vec<I::Item>> }
    impl<I: Iterator> Iterator for Rev<I> {
        type Item = I::Item;
        fn next(&mut self) -> Option<I::Item> {
            if self.buffer.is_none() {
                if crate::intrinsics::double_ended(&self.iter) { return crate::intrinsics::next_back(&mut self.iter); }
                self.buffer = Some(crate::intrinsics::iter_collect(&mut self.iter));
            }
            match &mut self.buffer {
                Some(items) => items.pop(),
                None => None,
            }
        }
        fn size_hint(&self) -> (usize, Option<usize>) {
            match &self.buffer {
                Some(items) => (items.len(), Some(items.len())),
                None => self.iter.size_hint(),
            }
        }
    }

    pub struct Empty<T> { items: Vec<T> }
    impl<T> Iterator for Empty<T> {
        type Item = T;
        fn size_hint(&self) -> (usize, Option<usize>) { (0, Some(0)) }
        fn next(&mut self) -> Option<T> { None }
    }
    pub fn empty<T>() -> Empty<T> { Empty { items: Vec::new() } }

    pub struct Once<T> { item: Option<T> }
    impl<T> Iterator for Once<T> {
        type Item = T;
        fn size_hint(&self) -> (usize, Option<usize>) {
            let n = if self.item.is_some() { 1 } else { 0 };
            (n, Some(n))
        }
        fn next(&mut self) -> Option<T> { self.item.take() }
    }
    pub fn once<T>(value: T) -> Once<T> { Once { item: Some(value) } }

    pub struct Repeat<T> { item: T }
    impl<T: Clone> Iterator for Repeat<T> {
        type Item = T;
        fn size_hint(&self) -> (usize, Option<usize>) { (usize::MAX, None) }
        fn next(&mut self) -> Option<T> { Some(self.item.clone()) }
    }
    pub fn repeat<T: Clone>(value: T) -> Repeat<T> { Repeat { item: value } }

    pub fn repeat_n<T: Clone>(value: T, count: usize) -> Take<Repeat<T>> { repeat(value).take(count) }

    pub struct FromFn<F> { f: F }
    impl<T, F: FnMut() -> Option<T>> Iterator for FromFn<F> {
        type Item = T;
        fn next(&mut self) -> Option<T> { (self.f)() }
    }
    pub fn from_fn<T, F: FnMut() -> Option<T>>(f: F) -> FromFn<F> { FromFn { f: f } }

    pub struct Successors<T, F> { next: Option<T>, f: F }
    impl<T, F: FnMut(&T) -> Option<T>> Iterator for Successors<T, F> {
        type Item = T;
        fn size_hint(&self) -> (usize, Option<usize>) {
            if self.next.is_some() { (1, None) } else { (0, Some(0)) }
        }
        fn next(&mut self) -> Option<T> {
            let item = self.next.take()?;
            self.next = (self.f)(&item);
            Some(item)
        }
    }
    pub fn successors<T, F: FnMut(&T) -> Option<T>>(first: Option<T>, succ: F) -> Successors<T, F> {
        Successors { next: first, f: succ }
    }

    pub struct RepeatWith<F> { f: F }
    impl<T, F: FnMut() -> T> Iterator for RepeatWith<F> {
        type Item = T;
        fn size_hint(&self) -> (usize, Option<usize>) { (usize::MAX, None) }
        fn next(&mut self) -> Option<T> { Some((self.f)()) }
    }
    pub fn repeat_with<T, F: FnMut() -> T>(f: F) -> RepeatWith<F> { RepeatWith { f: f } }

    pub trait DoubleEndedIterator: Iterator {
        fn next_back(&mut self) -> Option<Self::Item>;
        fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
            let mut remaining = n;
            while let Some(item) = self.next_back() {
                if remaining == 0 { return Some(item); }
                remaining -= 1;
            }
            None
        }
        fn rfold<B, F>(self, init: B, f: F) -> B where Self: Sized, F: FnMut(B, Self::Item) -> B {
            let mut iter = self;
            let mut f = f;
            let mut accumulator = init;
            while let Some(item) = iter.next_back() { accumulator = f(accumulator, item); }
            accumulator
        }
        fn rfind<P>(&mut self, predicate: P) -> Option<Self::Item> where Self: Sized, P: FnMut(&Self::Item) -> bool {
            let mut predicate = predicate;
            while let Some(item) = self.next_back() {
                if predicate(&item) { return Some(item); }
            }
            None
        }
    }

    pub trait ExactSizeIterator: Iterator {
        fn len(&self) -> usize {
            let (lower, _) = self.size_hint();
            lower
        }
    }

    pub trait FromIterator<A>: Sized {
        fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self;
    }

    pub trait Sum<A = Self>: Sized {
        fn sum<I: Iterator<Item = A>>(iter: I) -> Self;
    }

    pub trait Product<A = Self>: Sized {
        fn product<I: Iterator<Item = A>>(iter: I) -> Self;
    }

    impl<I: DoubleEndedIterator + ?Sized> DoubleEndedIterator for &mut I {
        fn next_back(&mut self) -> Option<I::Item> { (**self).next_back() }
    }
    impl<I: ExactSizeIterator + ?Sized> ExactSizeIterator for &mut I {}
    impl<T> DoubleEndedIterator for Lazy<T> {
        fn next_back(&mut self) -> Option<T> { crate::intrinsics::next_back(self) }
    }
    impl<I: DoubleEndedIterator> DoubleEndedIterator for Rev<I> {
        fn next_back(&mut self) -> Option<I::Item> { self.iter.next() }
    }
    impl<I: ExactSizeIterator + DoubleEndedIterator> ExactSizeIterator for Rev<I> {}
    impl<B, I: DoubleEndedIterator, F: FnMut(I::Item) -> B> DoubleEndedIterator for Map<I, F> {
        fn next_back(&mut self) -> Option<B> {
            match self.iter.next_back() {
                Some(item) => Some((self.f)(item)),
                None => None,
            }
        }
    }
    impl<B, I: ExactSizeIterator, F: FnMut(I::Item) -> B> ExactSizeIterator for Map<I, F> {}
    impl<I: DoubleEndedIterator, P: FnMut(&I::Item) -> bool> DoubleEndedIterator for Filter<I, P> {
        fn next_back(&mut self) -> Option<I::Item> {
            while let Some(item) = self.iter.next_back() {
                if (self.predicate)(&item) { return Some(item); }
            }
            None
        }
    }
    impl<B, I: DoubleEndedIterator, F: FnMut(I::Item) -> Option<B>> DoubleEndedIterator for FilterMap<I, F> {
        fn next_back(&mut self) -> Option<B> {
            while let Some(item) = self.iter.next_back() {
                if let Some(mapped) = (self.f)(item) { return Some(mapped); }
            }
            None
        }
    }
    impl<I: DoubleEndedIterator, F: FnMut(&I::Item)> DoubleEndedIterator for Inspect<I, F> {
        fn next_back(&mut self) -> Option<I::Item> {
            let item = self.iter.next_back()?;
            (self.f)(&item);
            Some(item)
        }
    }
    impl<I: ExactSizeIterator, F: FnMut(&I::Item)> ExactSizeIterator for Inspect<I, F> {}
    impl<A: DoubleEndedIterator, B: DoubleEndedIterator<Item = A::Item>> DoubleEndedIterator for Chain<A, B> {
        fn next_back(&mut self) -> Option<A::Item> {
            if let Some(item) = self.second.next_back() { return Some(item); }
            match &mut self.first {
                Some(first) => first.next_back(),
                None => None,
            }
        }
    }
    impl<I: DoubleEndedIterator + ExactSizeIterator> DoubleEndedIterator for Enumerate<I> {
        fn next_back(&mut self) -> Option<(usize, I::Item)> {
            let len = self.iter.len();
            let item = self.iter.next_back()?;
            Some((self.count + len - 1, item))
        }
    }
    impl<I: ExactSizeIterator> ExactSizeIterator for Enumerate<I> {}
    impl<A: DoubleEndedIterator + ExactSizeIterator, B: DoubleEndedIterator + ExactSizeIterator> DoubleEndedIterator for Zip<A, B> {
        fn next_back(&mut self) -> Option<(A::Item, B::Item)> {
            let a_len = self.a.len();
            let b_len = self.b.len();
            let mut extra = if a_len > b_len { a_len - b_len } else { 0 };
            while extra > 0 { self.a.next_back(); extra -= 1; }
            let mut extra = if b_len > a_len { b_len - a_len } else { 0 };
            while extra > 0 { self.b.next_back(); extra -= 1; }
            let left = self.a.next_back()?;
            let right = self.b.next_back()?;
            Some((left, right))
        }
    }
    impl<A: ExactSizeIterator, B: ExactSizeIterator> ExactSizeIterator for Zip<A, B> {}
    impl<I: DoubleEndedIterator + ExactSizeIterator> DoubleEndedIterator for Skip<I> {
        fn next_back(&mut self) -> Option<I::Item> {
            if self.len() > 0 { self.iter.next_back() } else { None }
        }
    }
    impl<I: ExactSizeIterator> ExactSizeIterator for Skip<I> {}
    impl<I: DoubleEndedIterator + ExactSizeIterator> DoubleEndedIterator for Take<I> {
        fn next_back(&mut self) -> Option<I::Item> {
            if self.n == 0 { return None; }
            let len = self.iter.len();
            let item = if len > self.n { self.iter.nth_back(len - self.n) } else { self.iter.next_back() };
            self.n -= 1;
            item
        }
    }
    impl<I: ExactSizeIterator> ExactSizeIterator for Take<I> {}
    impl<I: DoubleEndedIterator> DoubleEndedIterator for Fuse<I> {
        fn next_back(&mut self) -> Option<I::Item> {
            let item = match &mut self.iter {
                Some(iter) => iter.next_back(),
                None => return None,
            };
            if item.is_none() { self.iter = None; }
            item
        }
    }
    impl<I: ExactSizeIterator> ExactSizeIterator for Fuse<I> {}
    impl<I: DoubleEndedIterator> DoubleEndedIterator for Peekable<I> {
        fn next_back(&mut self) -> Option<I::Item> {
            match &mut self.peeked {
                Some(None) => None,
                Some(peeked) => match self.iter.next_back() {
                    Some(item) => Some(item),
                    None => peeked.take(),
                },
                None => self.iter.next_back(),
            }
        }
    }
    impl<I: ExactSizeIterator> ExactSizeIterator for Peekable<I> {}
    impl<T> DoubleEndedIterator for Once<T> {
        fn next_back(&mut self) -> Option<T> { self.item.take() }
    }
    impl<T> ExactSizeIterator for Once<T> {}
    impl<T> DoubleEndedIterator for Empty<T> {
        fn next_back(&mut self) -> Option<T> { None }
    }
    impl<T> ExactSizeIterator for Empty<T> {}
    impl<A: Copy> DoubleEndedIterator for crate::ops::Range<A> {
        fn next_back(&mut self) -> Option<A> { crate::intrinsics::range_next_back(self) }
    }
    impl<A: Copy> ExactSizeIterator for crate::ops::Range<A> {}
    impl<A: Copy> DoubleEndedIterator for crate::ops::RangeInclusive<A> {
        fn next_back(&mut self) -> Option<A> { crate::intrinsics::range_inclusive_next_back(self) }
    }
    impl<A: Copy> ExactSizeIterator for crate::ops::RangeInclusive<A> {}
    impl<I: Iterator + ?Sized> Iterator for Box<I> {
        type Item = I::Item;
        fn next(&mut self) -> Option<I::Item> { (**self).next() }
        fn size_hint(&self) -> (usize, Option<usize>) { (**self).size_hint() }
    }
    impl<I: DoubleEndedIterator + ?Sized> DoubleEndedIterator for Box<I> {
        fn next_back(&mut self) -> Option<I::Item> { (**self).next_back() }
    }
}

pub mod cmp {
    #[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord, Hash)]
    pub enum Ordering { Less = -1, Equal = 0, Greater = 1 }

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

    pub trait PartialEq<Rhs: ?Sized = Self> {
        fn eq(&self, other: &Rhs) -> bool;
        fn ne(&self, other: &Rhs) -> bool { !self.eq(other) }
    }

    pub trait Eq: PartialEq {}

    pub trait PartialOrd<Rhs: ?Sized = Self>: PartialEq<Rhs> {
        fn partial_cmp(&self, other: &Rhs) -> Option<Ordering>;
        fn lt(&self, other: &Rhs) -> bool { self.partial_cmp(other) == Some(Ordering::Less) }
        fn le(&self, other: &Rhs) -> bool {
            match self.partial_cmp(other) { Some(Ordering::Less) | Some(Ordering::Equal) => true, _ => false }
        }
        fn gt(&self, other: &Rhs) -> bool { self.partial_cmp(other) == Some(Ordering::Greater) }
        fn ge(&self, other: &Rhs) -> bool {
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
    pub fn max_by<T, F: FnOnce(&T, &T) -> Ordering>(v1: T, v2: T, compare: F) -> T {
        match compare(&v1, &v2) { Ordering::Greater => v1, _ => v2 }
    }
    pub fn min_by<T, F: FnOnce(&T, &T) -> Ordering>(v1: T, v2: T, compare: F) -> T {
        match compare(&v1, &v2) { Ordering::Greater => v2, _ => v1 }
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

    pub type Iter<'a, T> = Lazy<&'a T>;
    pub type IterMut<'a, T> = Lazy<&'a mut T>;
    pub type Windows<'a, T> = Lazy<&'a [T]>;
    pub type Chunks<'a, T> = Lazy<&'a [T]>;

    pub struct ChunksExact<'a, T> { chunks: Lazy<&'a [T]>, rem: &'a [T] }
    impl<'a, T> Iterator for ChunksExact<'a, T> {
        type Item = &'a [T];
        fn next(&mut self) -> Option<&'a [T]> { self.chunks.next() }
    }
    impl<'a, T> ChunksExact<'a, T> {
        pub fn remainder(&self) -> &'a [T] { self.rem }
    }

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
        pub fn split_at_mut(&mut self, mid: usize) -> (&mut [T], &mut [T]);
        pub fn split_first_mut(&mut self) -> Option<(&mut T, &mut [T])>;
        pub fn split_last_mut(&mut self) -> Option<(&mut T, &mut [T])>;
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
        pub fn chunks_exact(&self, size: usize) -> ChunksExact<'_, T> {
            let whole = self.len() - self.len() % size;
            ChunksExact { chunks: self[..whole].chunks(size), rem: &self[whole..] }
        }
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

    pub type IntoIter<T> = Lazy<T>;

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
        pub fn splice<R: crate::ops::RangeBounds, I: IntoIterator<Item = T>>(&mut self, range: R, replace_with: I) -> Lazy<T> {
            let (start, end) = crate::intrinsics::range_resolve(range, self.len());
            let tail = self.split_off(end);
            let removed = self.split_off(start);
            self.extend(replace_with);
            self.extend(tail);
            crate::intrinsics::lazy_elements(removed)
        }
        pub fn from_elem(elem: T, n: usize) -> Vec<T> where T: Clone { vec![elem; n] }
        pub fn from_iter<I: IntoIterator>(iter: I) -> Vec<T> {
            let mut items = Vec::new();
            items.extend(iter);
            items
        }
    }
}

pub mod mem {
    pub fn size_of<T>() -> usize;
    pub fn align_of<T>() -> usize;
    pub fn size_of_val<T: ?Sized>(value: &T) -> usize;
    pub fn discriminant<T>(value: &T) -> Discriminant;
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Discriminant(usize);

    pub fn replace<T>(dest: &mut T, src: T) -> T;
    pub fn swap<T>(x: &mut T, y: &mut T);
    pub fn take<T: Default>(dest: &mut T) -> T { replace(dest, T::default()) }
    pub fn drop<T>(_value: T) {}
    pub fn forget<T>(value: T);
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

    impl Option<String> {
        pub fn as_deref(&self) -> Option<&str> {
            match self { Some(value) => Some(value.as_str()), None => None }
        }
        pub fn as_deref_mut(&mut self) -> Option<&mut str> {
            match self { Some(value) => Some(value.as_mut_str()), None => None }
        }
    }
    impl<T> Option<Box<T>> {
        pub fn as_deref(&self) -> Option<&T> {
            match self { Some(value) => Some(&**value), None => None }
        }
        pub fn as_deref_mut(&mut self) -> Option<&mut T> {
            match self { Some(value) => Some(&mut **value), None => None }
        }
    }
    impl<T> Option<Vec<T>> {
        pub fn as_deref(&self) -> Option<&[T]> {
            match self { Some(value) => Some(value.as_slice()), None => None }
        }
        pub fn as_deref_mut(&mut self) -> Option<&mut [T]> {
            match self { Some(value) => Some(value.as_mut_slice()), None => None }
        }
    }
    impl Option<String> {
        pub fn as_deref_mut(&mut self) -> Option<&mut str> {
            match self { Some(value) => Some(value.as_mut_str()), None => None }
        }
    }
    impl<T> Option<Box<T>> {
        pub fn as_deref_mut(&mut self) -> Option<&mut T> {
            match self { Some(value) => Some(&mut **value), None => None }
        }
    }
    impl<T> Option<Vec<T>> {
        pub fn as_deref_mut(&mut self) -> Option<&mut [T]> {
            match self { Some(value) => Some(value.as_mut_slice()), None => None }
        }
    }

    impl<T> Option<Option<T>> {
        pub fn flatten(self) -> Option<T> {
            match self { Some(inner) => inner, None => None }
        }
    }

    impl<T, U> Option<(T, U)> {
        pub fn unzip(self) -> (Option<T>, Option<U>) {
            match self { Some((a, b)) => (Some(a), Some(b)), None => (None, None) }
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

    impl<T: Clone, E> Result<&T, E> {
        pub fn cloned(self) -> Result<T, E> {
            match self { Ok(value) => Ok(value.clone()), Err(err) => Err(err) }
        }
        pub fn copied(self) -> Result<T, E> {
            match self { Ok(value) => Ok(value.clone()), Err(err) => Err(err) }
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

    pub struct Formatter {
        buf: String,
        alternate: bool,
        width: Option<usize>,
        precision: Option<usize>,
        fill: char,
        align: u8,
        sign_plus: bool,
        sign_aware_zero_pad: bool,
        debug_hex: u8,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Alignment { Left, Right, Center }

    impl Formatter {
        pub fn width(&self) -> Option<usize> { self.width }
        pub fn precision(&self) -> Option<usize> { self.precision }
        pub fn fill(&self) -> char { self.fill }
        pub fn sign_plus(&self) -> bool { self.sign_plus }
        pub fn sign_minus(&self) -> bool { false }
        pub fn sign_aware_zero_pad(&self) -> bool { self.sign_aware_zero_pad }
        pub fn align(&self) -> Option<Alignment> {
            match self.align { 1 => Some(Alignment::Left), 2 => Some(Alignment::Right), 3 => Some(Alignment::Center), _ => None }
        }
        pub fn debug_struct(&mut self, name: &str) -> DebugStruct<'_> {
            self.buf.push_str(name);
            DebugStruct { fmt: self, fields: 0, closer: '}' }
        }
        pub fn debug_tuple(&mut self, name: &str) -> DebugTuple<'_> {
            self.buf.push_str(name);
            DebugTuple { inner: DebugStruct { fmt: self, fields: 0, closer: ')' } }
        }
        pub fn debug_list(&mut self) -> DebugList<'_> {
            self.buf.push_str("[");
            DebugList { inner: DebugStruct { fmt: self, fields: 0, closer: ']' } }
        }
        pub fn debug_set(&mut self) -> DebugList<'_> {
            self.buf.push_str("{");
            DebugList { inner: DebugStruct { fmt: self, fields: 0, closer: 's' } }
        }
        pub fn debug_map(&mut self) -> DebugMap<'_> {
            self.buf.push_str("{");
            DebugMap { inner: DebugStruct { fmt: self, fields: 0, closer: 'm' } }
        }
        pub fn write_str(&mut self, data: &str) -> Result {
            self.buf.push_str(data);
            Ok(())
        }
        pub fn write_fmt<A>(&mut self, text: A) -> Result {
            self.buf.push_str(&crate::intrinsics::arguments_text(text));
            Ok(())
        }
        pub fn write_char(&mut self, c: char) -> Result {
            self.buf.push(c);
            Ok(())
        }
        pub fn pad(&mut self, data: &str) -> Result {
            let text = crate::intrinsics::pad_text(data, self.width, self.precision, self.fill, self.align);
            self.buf.push_str(&text);
            Ok(())
        }
        pub fn alternate(&self) -> bool { self.alternate }
    }

    pub trait Display {
        fn fmt(&self, f: &mut Formatter) -> Result;
    }

    // One builder serves debug_struct, debug_tuple, debug_list, debug_set, and debug_map.
    pub struct DebugStruct<'a> { fmt: &'a mut Formatter, fields: usize, closer: char }
    pub struct DebugTuple<'a> { inner: DebugStruct<'a> }
    pub struct DebugList<'a> { inner: DebugStruct<'a> }
    pub type DebugSet<'a> = DebugList<'a>;
    pub struct DebugMap<'a> { inner: DebugStruct<'a> }

    impl<'a> DebugTuple<'a> {
        pub fn field(&mut self, value: &dyn Debug) -> &mut DebugTuple<'a> { self.inner.item(None, value); self }
        pub fn finish(&mut self) -> Result { self.inner.finish() }
    }
    impl<'a> DebugList<'a> {
        pub fn entry(&mut self, value: &dyn Debug) -> &mut DebugList<'a> { self.inner.item(None, value); self }
        pub fn entries<D: Debug, I: IntoIterator<Item = D>>(&mut self, entries: I) -> &mut DebugList<'a> {
            for entry in entries { self.inner.item(None, &entry); }
            self
        }
        pub fn finish(&mut self) -> Result { self.inner.finish() }
    }
    impl<'a> DebugMap<'a> {
        pub fn entry(&mut self, key: &dyn Debug, value: &dyn Debug) -> &mut DebugMap<'a> {
            let key = crate::intrinsics::debug_entry(key, self.inner.fmt);
            self.inner.item(Some(&key), value);
            self
        }
        pub fn entries<K: Debug, V: Debug, I: IntoIterator<Item = (K, V)>>(&mut self, entries: I) -> &mut DebugMap<'a> {
            for (key, value) in entries {
                let key = crate::intrinsics::debug_entry(&key, self.inner.fmt);
                self.inner.item(Some(&key), &value);
            }
            self
        }
        pub fn finish(&mut self) -> Result { self.inner.finish() }
    }

    impl<'a> DebugStruct<'a> {
        fn item(&mut self, name: Option<&str>, value: &dyn Debug) {
            let alternate = self.fmt.alternate;
            let named = self.closer == '}';
            let text = crate::intrinsics::debug_entry(value, self.fmt);
            let text = if alternate { text.replace("\n", "\n    ") } else { text };
            if alternate {
                if self.fields == 0 {
                    self.fmt.buf.push_str(if named { " {\n" } else if self.closer == ')' { "(\n" } else { "\n" });
                }
                self.fmt.buf.push_str("    ");
            } else if self.fields == 0 {
                self.fmt.buf.push_str(if named { " { " } else if self.closer == ')' { "(" } else { "" });
            } else {
                self.fmt.buf.push_str(", ");
            }
            if let Some(name) = name {
                self.fmt.buf.push_str(name);
                self.fmt.buf.push_str(": ");
            }
            self.fmt.buf.push_str(&text);
            if alternate { self.fmt.buf.push_str(",\n"); }
            self.fields += 1;
        }
        pub fn field(&mut self, name: &str, value: &dyn Debug) -> &mut DebugStruct<'a> {
            self.item(Some(name), value);
            self
        }
        pub fn finish_non_exhaustive(&mut self) -> Result {
            if self.fmt.alternate {
                self.fmt.buf.push_str(if self.fields == 0 { " {\n    ..\n}" } else { "    ..\n}" });
            } else {
                self.fmt.buf.push_str(if self.fields == 0 { " { .. }" } else { ", .. }" });
            }
            Ok(())
        }
        pub fn finish(&mut self) -> Result {
            let alternate = self.fmt.alternate && self.fields > 0;
            match self.closer {
                '}' => { if self.fields > 0 { self.fmt.buf.push_str(if alternate { "}" } else { " }" }); } }
                ')' => { if self.fields > 0 { self.fmt.buf.push_str(")"); } }
                ']' => self.fmt.buf.push_str("]"),
                _ => self.fmt.buf.push_str("}"),
            }
            Ok(())
        }
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
        fn write_fmt<A>(&mut self, text: A) -> Result {
            self.write_str(&crate::intrinsics::arguments_text(text))
        }
    }

    impl Write for String {
        fn write_str(&mut self, s: &str) -> Result {
            self.push_str(s);
            Ok(())
        }
    }

    // What format_args! builds: the formatted text, written verbatim by both Display and Debug.
    #[derive(Clone)]
    pub struct Arguments<'a> { text: String, life: crate::marker::PhantomData<&'a str> }
    impl<'a> Arguments<'a> {
        pub fn from_text(text: String) -> Arguments<'a> { Arguments { text, life: crate::marker::PhantomData } }
    }
    impl<'a> Display for Arguments<'a> {
        fn fmt(&self, f: &mut Formatter) -> Result { f.write_str(&self.text) }
    }
    impl<'a> Debug for Arguments<'a> {
        fn fmt(&self, f: &mut Formatter) -> Result { f.write_str(&self.text) }
    }

    pub trait LowerHex { fn fmt(&self, f: &mut Formatter) -> Result; }
    pub trait UpperHex { fn fmt(&self, f: &mut Formatter) -> Result; }
    pub trait Octal { fn fmt(&self, f: &mut Formatter) -> Result; }
    pub trait Binary { fn fmt(&self, f: &mut Formatter) -> Result; }
    pub trait LowerExp { fn fmt(&self, f: &mut Formatter) -> Result; }
    pub trait UpperExp { fn fmt(&self, f: &mut Formatter) -> Result; }

    macro_rules! integer_format_impls {
        ($($t:ident)*) => { $(
            impl LowerHex for $t { fn fmt(&self, f: &mut Formatter) -> Result { crate::intrinsics::kind_fmt(self, f, 'x') } }
            impl UpperHex for $t { fn fmt(&self, f: &mut Formatter) -> Result { crate::intrinsics::kind_fmt(self, f, 'X') } }
            impl Octal for $t { fn fmt(&self, f: &mut Formatter) -> Result { crate::intrinsics::kind_fmt(self, f, 'o') } }
            impl Binary for $t { fn fmt(&self, f: &mut Formatter) -> Result { crate::intrinsics::kind_fmt(self, f, 'b') } }
        )* };
    }
    integer_format_impls! { i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize }
    macro_rules! exponent_format_impls {
        ($($t:ident)*) => { $(
            impl LowerExp for $t { fn fmt(&self, f: &mut Formatter) -> Result { crate::intrinsics::kind_fmt(self, f, 'e') } }
            impl UpperExp for $t { fn fmt(&self, f: &mut Formatter) -> Result { crate::intrinsics::kind_fmt(self, f, 'E') } }
        )* };
    }
    exponent_format_impls! { i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize f32 f64 }
}

pub mod clone {
    pub trait Clone {
        fn clone(&self) -> Self;
        fn clone_from(&mut self, source: &Self) where Self: Sized { *self = source.clone(); }
    }
}

pub mod default {
    pub trait Default {
        fn default() -> Self;
    }
}

pub mod marker {
    pub trait Copy: Clone {}
    #[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct PhantomData<T: ?Sized>;
    impl<T: ?Sized> crate::fmt::Debug for PhantomData<T> {
        fn fmt(&self, f: &mut crate::fmt::Formatter) -> crate::fmt::Result {
            write!(f, "PhantomData<{}>", crate::any::type_name::<T>())
        }
    }
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
    pub trait Deref { type Target: ?Sized; fn deref(&self) -> &Self::Target; }
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
        pub fn len(&self) -> usize { crate::intrinsics::range_length(self) }
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
        fn size_hint(&self) -> (usize, Option<usize>) {
            let n = crate::intrinsics::range_count(self);
            (n, Some(n))
        }
        fn next(&mut self) -> Option<A> { crate::intrinsics::range_next(self) }
        fn nth(&mut self, n: usize) -> Option<A> { crate::intrinsics::range_nth(self, n) }
    }
    impl<A: Copy> Iterator for RangeInclusive<A> {
        type Item = A;
        fn size_hint(&self) -> (usize, Option<usize>) {
            let n = crate::intrinsics::range_count(self);
            (n, Some(n))
        }
        fn next(&mut self) -> Option<A> { crate::intrinsics::range_inclusive_next(self) }
    }
    impl<A: Copy> Iterator for RangeFrom<A> {
        type Item = A;
        fn size_hint(&self) -> (usize, Option<usize>) { (usize::MAX, None) }
        fn next(&mut self) -> Option<A> { crate::intrinsics::range_from_next(self) }
    }

    impl<Idx: fmt::Debug> fmt::Debug for Range<Idx> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::Debug::fmt(&self.start, f)?; f.write_str("..")?; fmt::Debug::fmt(&self.end, f) }
    }
    impl<Idx: fmt::Debug> fmt::Debug for RangeInclusive<Idx> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::Debug::fmt(&self.start, f)?; f.write_str("..=")?; fmt::Debug::fmt(&self.end, f) }
    }
    impl<Idx: fmt::Debug> fmt::Debug for RangeFrom<Idx> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::Debug::fmt(&self.start, f)?; f.write_str("..") }
    }
    impl<Idx: fmt::Debug> fmt::Debug for RangeTo<Idx> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { f.write_str("..")?; fmt::Debug::fmt(&self.end, f) }
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

pub mod num_wrapping {
    use crate::fmt;
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    pub struct Wrapping<T>(pub T);
    impl<T: fmt::Debug> fmt::Debug for Wrapping<T> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::Debug::fmt(&self.0, f) }
    }
    impl<T: fmt::Display> fmt::Display for Wrapping<T> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.0) }
    }
    impl<T> crate::ops::Add for Wrapping<T> {
        type Output = Wrapping<T>;
        fn add(self, other: Wrapping<T>) -> Wrapping<T> { Wrapping(crate::intrinsics::wrapping_op(self.0, other.0, 0)) }
    }
    impl<T> crate::ops::Sub for Wrapping<T> {
        type Output = Wrapping<T>;
        fn sub(self, other: Wrapping<T>) -> Wrapping<T> { Wrapping(crate::intrinsics::wrapping_op(self.0, other.0, 1)) }
    }
    impl<T> crate::ops::Mul for Wrapping<T> {
        type Output = Wrapping<T>;
        fn mul(self, other: Wrapping<T>) -> Wrapping<T> { Wrapping(crate::intrinsics::wrapping_op(self.0, other.0, 2)) }
    }
    impl<T> crate::ops::AddAssign for Wrapping<T> {
        fn add_assign(&mut self, other: Wrapping<T>) { self.0 = crate::intrinsics::wrapping_op(self.0, other.0, 0); }
    }
    impl<T> crate::ops::SubAssign for Wrapping<T> {
        fn sub_assign(&mut self, other: Wrapping<T>) { self.0 = crate::intrinsics::wrapping_op(self.0, other.0, 1); }
    }
    impl<T> crate::ops::MulAssign for Wrapping<T> {
        fn mul_assign(&mut self, other: Wrapping<T>) { self.0 = crate::intrinsics::wrapping_op(self.0, other.0, 2); }
    }
    impl<T> crate::ops::Div for Wrapping<T> {
        type Output = Wrapping<T>;
        fn div(self, other: Wrapping<T>) -> Wrapping<T> { Wrapping(crate::intrinsics::wrapping_op(self.0, other.0, 3)) }
    }
    impl<T> crate::ops::DivAssign for Wrapping<T> {
        fn div_assign(&mut self, other: Wrapping<T>) { self.0 = crate::intrinsics::wrapping_op(self.0, other.0, 3); }
    }
    impl<T> crate::ops::Rem for Wrapping<T> {
        type Output = Wrapping<T>;
        fn rem(self, other: Wrapping<T>) -> Wrapping<T> { Wrapping(crate::intrinsics::wrapping_op(self.0, other.0, 4)) }
    }
    impl<T> crate::ops::RemAssign for Wrapping<T> {
        fn rem_assign(&mut self, other: Wrapping<T>) { self.0 = crate::intrinsics::wrapping_op(self.0, other.0, 4); }
    }
    impl<T> crate::ops::BitAnd for Wrapping<T> {
        type Output = Wrapping<T>;
        fn bitand(self, other: Wrapping<T>) -> Wrapping<T> { Wrapping(crate::intrinsics::wrapping_op(self.0, other.0, 6)) }
    }
    impl<T> crate::ops::BitAndAssign for Wrapping<T> {
        fn bitand_assign(&mut self, other: Wrapping<T>) { self.0 = crate::intrinsics::wrapping_op(self.0, other.0, 6); }
    }
    impl<T> crate::ops::BitOr for Wrapping<T> {
        type Output = Wrapping<T>;
        fn bitor(self, other: Wrapping<T>) -> Wrapping<T> { Wrapping(crate::intrinsics::wrapping_op(self.0, other.0, 7)) }
    }
    impl<T> crate::ops::BitOrAssign for Wrapping<T> {
        fn bitor_assign(&mut self, other: Wrapping<T>) { self.0 = crate::intrinsics::wrapping_op(self.0, other.0, 7); }
    }
    impl<T> crate::ops::BitXor for Wrapping<T> {
        type Output = Wrapping<T>;
        fn bitxor(self, other: Wrapping<T>) -> Wrapping<T> { Wrapping(crate::intrinsics::wrapping_op(self.0, other.0, 8)) }
    }
    impl<T> crate::ops::BitXorAssign for Wrapping<T> {
        fn bitxor_assign(&mut self, other: Wrapping<T>) { self.0 = crate::intrinsics::wrapping_op(self.0, other.0, 8); }
    }
    impl<T: Copy> crate::ops::Neg for Wrapping<T> {
        type Output = Wrapping<T>;
        fn neg(self) -> Wrapping<T> { Wrapping(crate::intrinsics::wrapping_op(self.0, self.0, 5)) }
    }
    impl<T: Copy> crate::ops::Not for Wrapping<T> {
        type Output = Wrapping<T>;
        fn not(self) -> Wrapping<T> { Wrapping(crate::intrinsics::wrapping_op(self.0, self.0, 9)) }
    }
    impl<T> crate::ops::Shl<usize> for Wrapping<T> {
        type Output = Wrapping<T>;
        fn shl(self, amount: usize) -> Wrapping<T> { Wrapping(crate::intrinsics::wrapping_shift(self.0, amount, true)) }
    }
    impl<T> crate::ops::Shr<usize> for Wrapping<T> {
        type Output = Wrapping<T>;
        fn shr(self, amount: usize) -> Wrapping<T> { Wrapping(crate::intrinsics::wrapping_shift(self.0, amount, false)) }
    }
    impl<T> crate::ops::ShlAssign<usize> for Wrapping<T> {
        fn shl_assign(&mut self, amount: usize) { self.0 = crate::intrinsics::wrapping_shift(self.0, amount, true); }
    }
    impl<T> crate::ops::ShrAssign<usize> for Wrapping<T> {
        fn shr_assign(&mut self, amount: usize) { self.0 = crate::intrinsics::wrapping_shift(self.0, amount, false); }
    }
}

pub mod hash {
    pub trait Hasher {
        fn finish(&self) -> u64;
        fn write(&mut self, bytes: &[u8]);
        // As in std, each integer writes its native-endian (here little-endian) bytes.
        fn write_u8(&mut self, i: u8) { self.write(&[i]) }
        fn write_u16(&mut self, i: u16) { self.write(&i.to_le_bytes()) }
        fn write_u32(&mut self, i: u32) { self.write(&i.to_le_bytes()) }
        fn write_u64(&mut self, i: u64) { self.write(&i.to_le_bytes()) }
        fn write_u128(&mut self, i: u128) { self.write(&i.to_le_bytes()) }
        fn write_usize(&mut self, i: usize) { self.write(&i.to_le_bytes()) }
        fn write_i8(&mut self, i: i8) { self.write_u8(i as u8) }
        fn write_i16(&mut self, i: i16) { self.write_u16(i as u16) }
        fn write_i32(&mut self, i: i32) { self.write_u32(i as u32) }
        fn write_i64(&mut self, i: i64) { self.write_u64(i as u64) }
        fn write_i128(&mut self, i: i128) { self.write_u128(i as u128) }
        fn write_isize(&mut self, i: isize) { self.write_usize(i as usize) }
    }
    pub trait Hash {
        fn hash<H: Hasher>(&self, state: &mut H);
    }
    #[derive(Debug, Clone, Default)]
    pub struct DefaultHasher { state: u64 }
    impl DefaultHasher {
        pub fn new() -> DefaultHasher { DefaultHasher { state: 14695981039346656037 } }
    }
    impl Hasher for DefaultHasher {
        fn finish(&self) -> u64 { self.state }
        fn write(&mut self, bytes: &[u8]) {
            for byte in bytes { self.state = (self.state ^ (*byte as u64)).wrapping_mul(1099511628211); }
        }
        fn write_u64(&mut self, i: u64) { self.write(&i.to_le_bytes()) }
    }
    pub fn hash_of<T: Hash>(value: &T) -> u64 { crate::intrinsics::hash_value(value) }
    // Hash::hash for primitives and derived impls: feeds one structural hash into the hasher.
    pub fn hash_builtin<T, H: Hasher>(value: &T, state: &mut H) { state.write_u64(crate::intrinsics::hash_value(value)) }
    pub trait BuildHasher {
        type Hasher: Hasher;
        fn build_hasher(&self) -> Self::Hasher;
        fn hash_one<T: Hash>(&self, x: T) -> u64 {
            let mut state = self.build_hasher();
            x.hash(&mut state);
            state.finish()
        }
    }
}

pub mod str {
    use crate::iter::Lazy;

    pub type Lines<'a> = Lazy<&'a str>;
    pub type SplitWhitespace<'a> = Lazy<&'a str>;
    pub type Chars<'a> = Lazy<char>;
    pub type CharIndices<'a> = Lazy<(usize, char)>;
    pub type Bytes<'a> = Lazy<u8>;

    pub trait Pattern {}
    impl Pattern for char {}
    impl Pattern for &str {}
    impl Pattern for &&str {}
    impl Pattern for &String {}
    impl Pattern for &[char] {}
    impl<const N: usize> Pattern for [char; N] {}
    impl<const N: usize> Pattern for &[char; N] {}
    impl<F: FnMut(char) -> bool> Pattern for F {}

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Utf8Error { valid_up_to: usize }
    impl Utf8Error {
        pub fn valid_up_to(&self) -> usize { self.valid_up_to }
    }
    impl crate::fmt::Display for Utf8Error {
        fn fmt(&self, f: &mut crate::fmt::Formatter) -> crate::fmt::Result {
            write!(f, "invalid utf-8 sequence of 1 bytes from index {}", self.valid_up_to)
        }
    }
    pub fn from_utf8(v: &[u8]) -> Result<&str, Utf8Error>;

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
        pub fn escape_unicode(&self) -> String;
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
    impl<T: crate::fmt::Display + ?Sized> ToString for T {
        fn to_string(&self) -> String { format!("{}", self) }
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
        pub fn from_iter<I: IntoIterator>(iter: I) -> String {
            let mut text = String::new();
            text.extend(iter);
            text
        }
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
        pub fn escape_default(self) -> String;
        pub fn escape_debug(self) -> String;
        pub fn escape_unicode(self) -> String;
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TryFromCharError(());
    impl crate::fmt::Display for TryFromCharError {
        fn fmt(&self, f: &mut crate::fmt::Formatter) -> crate::fmt::Result { write!(f, "unicode code point out of range") }
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct CharTryFromError(pub(crate) ());
    impl crate::fmt::Display for CharTryFromError {
        fn fmt(&self, f: &mut crate::fmt::Formatter) -> crate::fmt::Result {
            write!(f, "converted integer out of range for `char`")
        }
    }
    impl crate::error::Error for CharTryFromError {}
    impl crate::convert::TryFrom<char> for u8 {
        type Error = TryFromCharError;
        fn try_from(c: char) -> Result<u8, TryFromCharError> {
            if (c as u32) < 256 { Ok(c as u8) } else { Err(TryFromCharError(())) }
        }
    }
    impl crate::convert::TryFrom<char> for u16 {
        type Error = TryFromCharError;
        fn try_from(c: char) -> Result<u16, TryFromCharError> {
            if (c as u32) < 65536 { Ok(c as u32 as u16) } else { Err(TryFromCharError(())) }
        }
    }
    impl crate::convert::From<char> for u128 { fn from(value: char) -> u128 { value as u128 } }
    pub fn from_u32(i: u32) -> Option<char> { char::from_u32(i) }
    pub fn from_digit(num: u32, radix: u32) -> Option<char> { char::from_digit(num, radix) }
}

pub mod f64 {
    pub const MAX: f64 = f64::MAX;
    pub const MIN: f64 = f64::MIN;
    pub const MIN_POSITIVE: f64 = f64::MIN_POSITIVE;
    pub const EPSILON: f64 = f64::EPSILON;
    pub const NAN: f64 = f64::NAN;
    pub const INFINITY: f64 = f64::INFINITY;
    pub const NEG_INFINITY: f64 = f64::NEG_INFINITY;
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
    pub const MAX: f32 = f32::MAX;
    pub const MIN: f32 = f32::MIN;
    pub const MIN_POSITIVE: f32 = f32::MIN_POSITIVE;
    pub const EPSILON: f32 = f32::EPSILON;
    pub const NAN: f32 = f32::NAN;
    pub const INFINITY: f32 = f32::INFINITY;
    pub const NEG_INFINITY: f32 = f32::NEG_INFINITY;
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
    pub fn identity<T>(x: T) -> T { x }

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
    impl From<String> for Box<str> { fn from(value: String) -> Box<str> { crate::intrinsics::box_unsized(value) } }
    impl From<&str> for Box<str> { fn from(value: &str) -> Box<str> { crate::intrinsics::box_unsized(value.to_string()) } }
    impl<T> From<Vec<T>> for Box<[T]> { fn from(value: Vec<T>) -> Box<[T]> { crate::intrinsics::box_unsized(value) } }
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
    impl From<bool> for i8 { fn from(value: bool) -> i8 { value as i8 } }
    impl From<bool> for i16 { fn from(value: bool) -> i16 { value as i16 } }
    impl From<bool> for u16 { fn from(value: bool) -> u16 { value as u16 } }
    impl From<bool> for i128 { fn from(value: bool) -> i128 { value as i128 } }
    impl From<bool> for u128 { fn from(value: bool) -> u128 { value as u128 } }
    impl From<bool> for isize { fn from(value: bool) -> isize { value as isize } }
    impl From<f32> for f64 { fn from(value: f32) -> f64 { value as f64 } }
    impl TryFrom<u32> for char {
        type Error = crate::char::CharTryFromError;
        fn try_from(value: u32) -> Result<char, crate::char::CharTryFromError> {
            match char::from_u32(value) { Some(c) => Ok(c), None => Err(crate::char::CharTryFromError(())) }
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
        pub use super::{Entry, OccupiedEntry, VacantEntry};
        pub use crate::hash::DefaultHasher;
    }
    pub mod btree_map {
        pub use super::BTreeMap;
        pub use super::{Entry, OccupiedEntry, VacantEntry};
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

    pub enum Entry<'a, M, K, V> {
        Occupied(OccupiedEntry<'a, M, K, V>),
        Vacant(VacantEntry<'a, M, K, V>),
    }
    pub struct OccupiedEntry<'a, M, K, V> { map: &'a mut M, key: K, value: crate::marker::PhantomData<V> }
    pub struct VacantEntry<'a, M, K, V> { map: &'a mut M, key: K, value: crate::marker::PhantomData<V> }

    impl<'a, M, K, V> Entry<'a, M, K, V> {
        fn new(map: &'a mut M, key: K) -> Entry<'a, M, K, V> {
            if crate::intrinsics::map_has(&*map, &key) {
                Entry::Occupied(OccupiedEntry { map: map, key: key, value: crate::marker::PhantomData })
            } else {
                Entry::Vacant(VacantEntry { map: map, key: key, value: crate::marker::PhantomData })
            }
        }
        pub fn or_insert(self, default: V) -> &'a mut V {
            match self { Entry::Occupied(entry) => entry.into_mut(), Entry::Vacant(entry) => entry.insert(default) }
        }
        pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
            match self { Entry::Occupied(entry) => entry.into_mut(), Entry::Vacant(entry) => entry.insert(default()) }
        }
        pub fn or_insert_with_key<F: FnOnce(&K) -> V>(self, default: F) -> &'a mut V {
            match self {
                Entry::Occupied(entry) => entry.into_mut(),
                Entry::Vacant(entry) => {
                    let value = default(&entry.key);
                    entry.insert(value)
                }
            }
        }
        pub fn or_default(self) -> &'a mut V where V: Default {
            match self { Entry::Occupied(entry) => entry.into_mut(), Entry::Vacant(entry) => entry.insert(V::default()) }
        }
        pub fn and_modify<F: FnOnce(&mut V)>(self, f: F) -> Entry<'a, M, K, V> {
            match self {
                Entry::Occupied(mut entry) => {
                    f(entry.get_mut());
                    Entry::Occupied(entry)
                }
                Entry::Vacant(entry) => Entry::Vacant(entry),
            }
        }
        pub fn key(&self) -> &K {
            match self { Entry::Occupied(entry) => &entry.key, Entry::Vacant(entry) => &entry.key }
        }
    }

    impl<'a, M, K, V> OccupiedEntry<'a, M, K, V> {
        pub fn key(&self) -> &K { &self.key }
        pub fn get(&self) -> &V { crate::intrinsics::map_slot(self.map, &self.key) }
        pub fn get_mut(&mut self) -> &mut V { crate::intrinsics::map_slot(self.map, &self.key) }
        pub fn into_mut(self) -> &'a mut V { crate::intrinsics::map_slot(self.map, &self.key) }
        pub fn insert(&mut self, value: V) -> V { crate::mem::replace(self.get_mut(), value) }
        pub fn remove(self) -> V {
            let removed: Option<V> = crate::intrinsics::map_remove(self.map, &self.key);
            removed.unwrap()
        }
        pub fn remove_entry(self) -> (K, V) {
            let removed: Option<(K, V)> = crate::intrinsics::map_remove_entry(self.map, &self.key);
            removed.unwrap()
        }
    }

    impl<'a, M, K, V> VacantEntry<'a, M, K, V> {
        pub fn key(&self) -> &K { &self.key }
        pub fn into_key(self) -> K { self.key }
        pub fn insert(self, value: V) -> &'a mut V { crate::intrinsics::map_or_insert(self.map, self.key, value) }
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
                pub fn entry(&mut self, key: K) -> Entry<'_, $map<K, V>, K, V> { Entry::new(self, key) }
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
                type Item = (K, V);
                type IntoIter = Lazy<(K, V)>;
                fn into_iter(self) -> Lazy<(K, V)> { crate::intrinsics::map_items(self) }
            }
            impl<K, V> IntoIterator for &$map<K, V> {
                type Item = (&K, &V);
                type IntoIter = Lazy<(&K, &V)>;
                fn into_iter(self) -> Lazy<(&K, &V)> { crate::intrinsics::map_items(self) }
            }
            impl<K, V> IntoIterator for &mut $map<K, V> {
                type Item = (&K, &mut V);
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
            impl<K, V, const N: usize> From<[(K, V); N]> for $map<K, V> {
                fn from(entries: [(K, V); N]) -> $map<K, V> {
                    let mut map = $map::new();
                    map.extend(entries);
                    map
                }
            }
            impl<K, V> $map<K, V> {
                pub fn append(&mut self, other: &mut $map<K, V>) { self.extend(other.drain()) }
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
                type Item = T;
                type IntoIter = Lazy<T>;
                fn into_iter(self) -> Lazy<T> { crate::intrinsics::map_keys(self) }
            }
            impl<T> IntoIterator for &$set<T> {
                type Item = &T;
                type IntoIter = Lazy<&T>;
                fn into_iter(self) -> Lazy<&T> { crate::intrinsics::map_keys(self) }
            }
            impl<T: fmt::Debug> fmt::Debug for $set<T> {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { crate::intrinsics::map_debug(self, f) }
            }
            impl<T, const N: usize> From<[T; N]> for $set<T> {
                fn from(items: [T; N]) -> $set<T> {
                    let mut set = $set::new();
                    set.extend(items);
                    set
                }
            }
            impl<T> $set<T> {
                pub fn append(&mut self, other: &mut $set<T>) { self.extend(other.drain()) }
            }
            impl<T: Clone> crate::ops::BitOr<&$set<T>> for &$set<T> {
                type Output = $set<T>;
                fn bitor(self, rhs: &$set<T>) -> $set<T> { self.union(rhs).cloned().collect() }
            }
            impl<T: Clone> crate::ops::BitAnd<&$set<T>> for &$set<T> {
                type Output = $set<T>;
                fn bitand(self, rhs: &$set<T>) -> $set<T> { self.intersection(rhs).cloned().collect() }
            }
            impl<T: Clone> crate::ops::Sub<&$set<T>> for &$set<T> {
                type Output = $set<T>;
                fn sub(self, rhs: &$set<T>) -> $set<T> { self.difference(rhs).cloned().collect() }
            }
            impl<T: Clone> crate::ops::BitXor<&$set<T>> for &$set<T> {
                type Output = $set<T>;
                fn bitxor(self, rhs: &$set<T>) -> $set<T> { self.symmetric_difference(rhs).cloned().collect() }
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
        pub fn retain_mut<F: FnMut(&mut T) -> bool>(&mut self, f: F) { self.items.retain_mut(f) }
        pub fn binary_search(&self, x: &T) -> Result<usize, usize> where T: Ord { self.items.binary_search(x) }
        pub fn binary_search_by<F: FnMut(&T) -> crate::cmp::Ordering>(&self, f: F) -> Result<usize, usize> { self.items.binary_search_by(f) }
        pub fn binary_search_by_key<B: Ord, F: FnMut(&T) -> B>(&self, b: &B, f: F) -> Result<usize, usize> { self.items.binary_search_by_key(b, f) }
        pub fn partition_point<P: FnMut(&T) -> bool>(&self, pred: P) -> usize { self.items.partition_point(pred) }
        pub fn swap_remove_back(&mut self, index: usize) -> Option<T> { if index < self.items.len() { Some(self.items.swap_remove(index)) } else { None } }
        pub fn shrink_to_fit(&mut self) {}
        pub fn reserve(&mut self, additional: usize) {}
    }
    impl<T> Index<usize> for VecDeque<T> {
        type Output = T;
        fn index(&self, index: usize) -> &T { &self.items[index] }
    }
    impl<T> IndexMut<usize> for VecDeque<T> {
        fn index_mut(&mut self, index: usize) -> &mut T { &mut self.items[index] }
    }
    impl<T> IntoIterator for VecDeque<T> {
        type Item = T;
        type IntoIter = Lazy<T>;
        fn into_iter(self) -> Lazy<T> { crate::intrinsics::lazy_elements(self.items) }
    }
    impl<T> IntoIterator for &VecDeque<T> {
        type Item = &T;
        type IntoIter = Lazy<&T>;
        fn into_iter(self) -> Lazy<&T> { crate::intrinsics::lazy_elements(&self.items) }
    }
    impl<T: fmt::Debug> fmt::Debug for VecDeque<T> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::Debug::fmt(&self.items, f) }
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
        type Item = T;
        type IntoIter = Lazy<T>;
        fn into_iter(self) -> Lazy<T> { crate::intrinsics::lazy_elements(self.items) }
    }
    impl<T: fmt::Debug> fmt::Debug for BinaryHeap<T> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::Debug::fmt(&self.items, f) }
    }
}

pub mod rc {
    use crate::fmt;
    use crate::ops::Deref;

    // At runtime an Rc is a shared cell with strong and weak counts.
    pub struct Rc<T: ?Sized> { value: Box<T> }
    pub struct Weak<T: ?Sized> { value: Option<Box<T>> }

    impl<T: ?Sized> Rc<T> {
        pub fn new(value: T) -> Rc<T>;
        pub fn clone(this: &Rc<T>) -> Rc<T>;
        pub fn strong_count(this: &Rc<T>) -> usize;
        pub fn weak_count(this: &Rc<T>) -> usize;
        pub fn ptr_eq(this: &Rc<T>, other: &Rc<T>) -> bool;
        pub fn downgrade(this: &Rc<T>) -> Weak<T>;
        pub fn try_unwrap(this: Rc<T>) -> Result<T, Rc<T>>;
        pub fn into_inner(this: Rc<T>) -> Option<T>;
        pub fn unwrap_or_clone(this: Rc<T>) -> T where T: Clone;
        pub fn get_mut(this: &mut Rc<T>) -> Option<&mut T>;
        pub fn make_mut(this: &mut Rc<T>) -> &mut T where T: Clone;
        pub fn as_ptr(this: &Rc<T>) -> usize;
    }
    impl<T: ?Sized> Deref for Rc<T> {
        type Target = T;
        fn deref(&self) -> &T { crate::intrinsics::rc_deref(self) }
    }
    impl<T> AsRef<T> for Rc<T> {
        fn as_ref(&self) -> &T { crate::intrinsics::rc_deref(self) }
    }
    impl<T> From<T> for Rc<T> {
        fn from(value: T) -> Rc<T> { Rc::new(value) }
    }
    impl<T: ?Sized> Weak<T> {
        pub fn new() -> Weak<T>;
        pub fn upgrade(&self) -> Option<Rc<T>>;
        pub fn strong_count(&self) -> usize;
        pub fn weak_count(&self) -> usize;
    }
}

pub mod sync {
    pub use crate::rc::Rc as Arc;
    pub use crate::rc::Weak;

    pub mod mpsc {
        use crate::rc::Rc;
        use crate::cell::RefCell;
        use crate::collections::VecDeque;

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct RecvError;
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum TryRecvError { Empty, Disconnected }
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct SendError<T>(pub T);

        pub struct Sender<T> { queue: Rc<RefCell<VecDeque<T>>> }
        pub struct Receiver<T> { queue: Rc<RefCell<VecDeque<T>>> }
        pub type SyncSender<T> = Sender<T>;

        impl<T> Clone for Sender<T> {
            fn clone(&self) -> Sender<T> { Sender { queue: Rc::clone(&self.queue) } }
        }
        pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
            let queue = Rc::new(RefCell::new(VecDeque::new()));
            (Sender { queue: Rc::clone(&queue) }, Receiver { queue: queue })
        }
        pub fn sync_channel<T>(bound: usize) -> (Sender<T>, Receiver<T>) { channel() }
        impl<T> Sender<T> {
            pub fn send(&self, value: T) -> Result<(), SendError<T>> { self.queue.borrow_mut().push_back(value); Ok(()) }
        }
        impl<T> Receiver<T> {
            pub fn recv(&self) -> Result<T, RecvError> {
                match self.queue.borrow_mut().pop_front() { Some(value) => Ok(value), None => Err(RecvError) }
            }
            pub fn try_recv(&self) -> Result<T, TryRecvError> {
                match self.queue.borrow_mut().pop_front() { Some(value) => Ok(value), None => Err(TryRecvError::Disconnected) }
            }
            pub fn recv_timeout(&self, timeout: crate::time::Duration) -> Result<T, RecvError> { self.recv() }
            pub fn iter(&self) -> Lazy<T> { crate::intrinsics::lazy_elements(self.queue.borrow_mut().drain(..).collect::<Vec<T>>()) }
            pub fn try_iter(&self) -> Lazy<T> { self.iter() }
        }
        use crate::iter::Lazy;
        impl<T> IntoIterator for Receiver<T> {
            type Item = T;
            type IntoIter = Lazy<T>;
            fn into_iter(self) -> Lazy<T> { self.iter() }
        }
        impl<T> IntoIterator for &Receiver<T> {
            type Item = T;
            type IntoIter = Lazy<T>;
            fn into_iter(self) -> Lazy<T> { self.iter() }
        }
    }

    pub mod atomic {
        use crate::cell::Cell;
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Ordering { Relaxed, Release, Acquire, AcqRel, SeqCst }
        macro_rules! atomic {
            ($name:ident, $t:ty) => {
                #[derive(Default)]
                pub struct $name { value: Cell<$t> }
                impl $name {
                    pub fn new(value: $t) -> $name { $name { value: Cell::new(value) } }
                    pub fn load(&self, order: Ordering) -> $t { self.value.get() }
                    pub fn store(&self, value: $t, order: Ordering) { self.value.set(value) }
                    pub fn swap(&self, value: $t, order: Ordering) -> $t { self.value.replace(value) }
                    pub fn into_inner(self) -> $t { self.value.get() }
                    pub fn get_mut(&mut self) -> &mut $t { self.value.get_mut() }
                    pub fn compare_exchange(&self, current: $t, new: $t, success: Ordering, failure: Ordering) -> Result<$t, $t> {
                        let old = self.value.get();
                        if old == current { self.value.set(new); Ok(old) } else { Err(old) }
                    }
                }
                impl crate::fmt::Debug for $name {
                    fn fmt(&self, f: &mut crate::fmt::Formatter) -> crate::fmt::Result { write!(f, "{:?}", self.value.get()) }
                }
            };
        }
        macro_rules! atomic_int {
            ($name:ident, $t:ty) => {
                impl $name {
                    pub fn fetch_add(&self, value: $t, order: Ordering) -> $t { let old = self.value.get(); self.value.set(old.wrapping_add(value)); old }
                    pub fn fetch_sub(&self, value: $t, order: Ordering) -> $t { let old = self.value.get(); self.value.set(old.wrapping_sub(value)); old }
                    pub fn fetch_max(&self, value: $t, order: Ordering) -> $t { let old = self.value.get(); if value > old { self.value.set(value); } old }
                    pub fn fetch_min(&self, value: $t, order: Ordering) -> $t { let old = self.value.get(); if value < old { self.value.set(value); } old }
                }
            };
        }
        atomic!(AtomicUsize, usize);
        atomic_int!(AtomicUsize, usize);
        atomic!(AtomicIsize, isize);
        atomic_int!(AtomicIsize, isize);
        atomic!(AtomicU8, u8);
        atomic_int!(AtomicU8, u8);
        atomic!(AtomicU16, u16);
        atomic_int!(AtomicU16, u16);
        atomic!(AtomicU32, u32);
        atomic_int!(AtomicU32, u32);
        atomic!(AtomicU64, u64);
        atomic_int!(AtomicU64, u64);
        atomic!(AtomicI8, i8);
        atomic_int!(AtomicI8, i8);
        atomic!(AtomicI16, i16);
        atomic_int!(AtomicI16, i16);
        atomic!(AtomicI32, i32);
        atomic_int!(AtomicI32, i32);
        atomic!(AtomicI64, i64);
        atomic_int!(AtomicI64, i64);
        atomic!(AtomicBool, bool);
        impl AtomicBool {
            pub fn fetch_or(&self, value: bool, order: Ordering) -> bool { let old = self.value.get(); self.value.set(old || value); old }
            pub fn fetch_and(&self, value: bool, order: Ordering) -> bool { let old = self.value.get(); self.value.set(old && value); old }
        }
    }

    pub struct Mutex<T> { value: crate::cell::RefCell<T> }
    pub struct RwLock<T> { value: crate::cell::RefCell<T> }
    #[derive(Debug)]
    pub struct PoisonError<T> { guard: T }
    pub type LockResult<T> = Result<T, PoisonError<T>>;

    impl<T> Mutex<T> {
        pub fn new(value: T) -> Mutex<T> { Mutex { value: crate::cell::RefCell::new(value) } }
        pub fn lock(&self) -> Result<&mut T, PoisonError<&mut T>> { Ok(crate::intrinsics::refcell_value(&self.value)) }
        pub fn try_lock(&self) -> Result<&mut T, PoisonError<&mut T>> { Ok(crate::intrinsics::refcell_value(&self.value)) }
        pub fn into_inner(self) -> Result<T, PoisonError<T>> { Ok(self.value.into_inner()) }
        pub fn get_mut(&mut self) -> Result<&mut T, PoisonError<&mut T>> { Ok(self.value.get_mut()) }
    }
    impl<T> RwLock<T> {
        pub fn new(value: T) -> RwLock<T> { RwLock { value: crate::cell::RefCell::new(value) } }
        pub fn read(&self) -> Result<&T, PoisonError<&T>> { Ok(crate::intrinsics::refcell_value(&self.value)) }
        pub fn write(&self) -> Result<&mut T, PoisonError<&mut T>> { Ok(crate::intrinsics::refcell_value(&self.value)) }
        pub fn into_inner(self) -> Result<T, PoisonError<T>> { Ok(self.value.into_inner()) }
    }
}

pub mod cell {
    use crate::fmt;

    // `borrow` counts shared borrows above zero and marks a mutable borrow with -1.
    pub struct RefCell<T: ?Sized> { value: T, borrow: Cell<isize> }

    pub struct Ref<'b, T: ?Sized> { value: &'b T, borrow: &'b Cell<isize> }
    pub struct RefMut<'b, T: ?Sized> { value: &'b mut T, borrow: &'b Cell<isize> }

    impl<'b, T: ?Sized> crate::ops::Deref for Ref<'b, T> {
        type Target = T;
        fn deref(&self) -> &T { self.value }
    }
    impl<'b, T: ?Sized> Drop for Ref<'b, T> {
        fn drop(&mut self) { self.borrow.set(self.borrow.get() - 1); }
    }
    impl<'b, T: ?Sized> crate::ops::Deref for RefMut<'b, T> {
        type Target = T;
        fn deref(&self) -> &T { &*self.value }
    }
    impl<'b, T: ?Sized> crate::ops::DerefMut for RefMut<'b, T> {
        fn deref_mut(&mut self) -> &mut T { &mut *self.value }
    }
    impl<'b, T: ?Sized> Drop for RefMut<'b, T> {
        fn drop(&mut self) { self.borrow.set(0); }
    }
    impl<'b, T: fmt::Display> fmt::Display for Ref<'b, T> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::Display::fmt(self.value, f) }
    }
    impl<'b, T: fmt::Debug> fmt::Debug for Ref<'b, T> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::Debug::fmt(self.value, f) }
    }
    impl<'b, T: fmt::Display> fmt::Display for RefMut<'b, T> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::Display::fmt(&*self.value, f) }
    }
    impl<'b, T: fmt::Debug> fmt::Debug for RefMut<'b, T> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::Debug::fmt(&*self.value, f) }
    }

    impl<T: Clone> Clone for RefCell<T> {
        fn clone(&self) -> RefCell<T> { RefCell::new(self.borrow().clone()) }
    }
    impl<T: Default> Default for RefCell<T> {
        fn default() -> RefCell<T> { RefCell::new(T::default()) }
    }
    impl<T: PartialEq> PartialEq for RefCell<T> {
        fn eq(&self, other: &RefCell<T>) -> bool { *self.borrow() == *other.borrow() }
    }
    #[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Cell<T> { value: T }

    impl<T> RefCell<T> {
        pub fn new(value: T) -> RefCell<T> {
            crate::intrinsics::track_drops();
            RefCell { value: value, borrow: Cell::new(0) }
        }
    }
    // Borrowing works for unsized contents too, as in `Rc<RefCell<dyn Trait>>`.
    impl<T: ?Sized> RefCell<T> {
        pub fn borrow(&self) -> Ref<'_, T> {
            match self.try_borrow() {
                Ok(guard) => guard,
                Err(_) => panic!("already mutably borrowed: BorrowError"),
            }
        }
        pub fn borrow_mut(&self) -> RefMut<'_, T> {
            match self.try_borrow_mut() {
                Ok(guard) => guard,
                Err(_) => panic!("already borrowed: BorrowMutError"),
            }
        }
        pub fn try_borrow(&self) -> Result<Ref<'_, T>, BorrowError> {
            let state = self.borrow.get();
            if state < 0 { return Err(BorrowError); }
            self.borrow.set(state + 1);
            Ok(Ref { value: &*crate::intrinsics::refcell_value(self), borrow: &self.borrow })
        }
        pub fn try_borrow_mut(&self) -> Result<RefMut<'_, T>, BorrowMutError> {
            if self.borrow.get() != 0 { return Err(BorrowMutError); }
            self.borrow.set(-1);
            Ok(RefMut { value: crate::intrinsics::refcell_value(self), borrow: &self.borrow })
        }
    }
    impl<T> RefCell<T> {
        pub fn get_mut(&mut self) -> &mut T { &mut self.value }
        pub fn into_inner(self) -> T { self.value }
        pub fn replace(&self, value: T) -> T { crate::mem::replace(crate::intrinsics::refcell_value(self), value) }
        pub fn replace_with<F: FnOnce(&mut T) -> T>(&self, f: F) -> T {
            let slot: &mut T = crate::intrinsics::refcell_value(self);
            let value = f(slot);
            crate::mem::replace(slot, value)
        }
        pub fn take(&self) -> T where T: Default { crate::mem::take(crate::intrinsics::refcell_value(self)) }
        pub fn swap(&self, other: &RefCell<T>) {
            crate::mem::swap(crate::intrinsics::refcell_value(self), crate::intrinsics::refcell_value(other))
        }
    }
    impl<T: fmt::Debug> fmt::Debug for RefCell<T> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { f.debug_struct("RefCell").field("value", &self.value).finish() }
    }

    impl<T> Cell<T> {
        pub fn new(value: T) -> Cell<T> { Cell { value: value } }
        pub fn get(&self) -> T where T: Copy { self.value }
        pub fn set(&self, value: T) { *crate::intrinsics::cell_value(self) = value; }
        pub fn replace(&self, value: T) -> T { crate::mem::replace(crate::intrinsics::cell_value(self), value) }
        pub fn take(&self) -> T where T: Default { crate::mem::take(crate::intrinsics::cell_value(self)) }
        pub fn get_mut(&mut self) -> &mut T { &mut self.value }
        pub fn into_inner(self) -> T { self.value }
        pub fn update<F: FnOnce(T) -> T>(&self, f: F) -> T where T: Copy {
            let value = f(self.value);
            self.set(value);
            value
        }
        pub fn swap(&self, other: &Cell<T>) {
            crate::mem::swap(crate::intrinsics::cell_value(self), crate::intrinsics::cell_value(other))
        }
    }
    impl<T: fmt::Debug + Copy> fmt::Debug for Cell<T> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { f.debug_struct("Cell").field("value", &self.value).finish() }
    }

    #[derive(Debug)]
    pub struct BorrowError;
    #[derive(Debug)]
    pub struct BorrowMutError;
}

pub mod thread {
    // Threads run to completion when spawned, which is one valid interleaving.
    pub struct JoinHandle<T> { result: Option<T> }
    impl<T> JoinHandle<T> {
        pub fn join(self) -> Result<T, Box<dyn crate::any::Any>> {
            match self.result { Some(value) => Ok(value), None => panic!("thread result already taken") }
        }
        pub fn is_finished(&self) -> bool { true }
        pub fn thread(&self) -> Thread { Thread { name: None } }
    }
    pub fn spawn<T, F: FnOnce() -> T>(f: F) -> JoinHandle<T> { JoinHandle { result: Some(f()) } }
    pub fn sleep(duration: crate::time::Duration) {}
    pub fn yield_now() {}
    pub struct Thread { name: Option<String> }
    impl Thread {
        pub fn name(&self) -> Option<&str> { None }
        pub fn id(&self) -> ThreadId { ThreadId(1) }
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ThreadId(u64);
    pub fn current() -> Thread { Thread { name: Some(String::from("main")) } }
    pub struct Scope { marker: () }
    pub struct ScopedJoinHandle<T> { result: Option<T> }
    impl<T> ScopedJoinHandle<T> {
        pub fn join(self) -> Result<T, Box<dyn crate::any::Any>> {
            match self.result { Some(value) => Ok(value), None => panic!("thread result already taken") }
        }
    }
    impl Scope {
        pub fn spawn<T, F: FnOnce() -> T>(&self, f: F) -> ScopedJoinHandle<T> { ScopedJoinHandle { result: Some(f()) } }
    }
    pub fn scope<T, F: FnOnce(&Scope) -> T>(f: F) -> T { f(&Scope { marker: () }) }
}

pub mod any {
    #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
    pub struct TypeId { name: &'static str }
    impl TypeId {
        pub fn of<T: ?Sized + 'static>() -> TypeId { TypeId { name: type_name::<T>() } }
    }
    pub trait Any { fn type_id(&self) -> TypeId; }
    impl<T: ?Sized + 'static> Any for T {
        fn type_id(&self) -> TypeId { TypeId::of::<T>() }
    }
    impl dyn Any {
        pub fn is<T: Any>(&self) -> bool { self.downcast_ref::<T>().is_some() }
        pub fn downcast_ref<T: Any>(&self) -> Option<&T>;
        pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T>;
    }
    impl Box<dyn Any> {
        pub fn downcast<T: Any>(self) -> Result<Box<T>, Box<dyn Any>>;
    }
    pub fn type_name<T: ?Sized>() -> &'static str;
    pub fn type_name_of_val<T: ?Sized>(value: &T) -> &'static str { type_name::<T>() }
}

pub mod time {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    pub struct Duration { nanos: u128 }
    impl Duration {
        pub const ZERO: Duration = Duration { nanos: 0 };
        pub fn from_secs(secs: u64) -> Duration { Duration { nanos: secs as u128 * 1_000_000_000 } }
        pub fn from_millis(millis: u64) -> Duration { Duration { nanos: millis as u128 * 1_000_000 } }
        pub fn from_micros(micros: u64) -> Duration { Duration { nanos: micros as u128 * 1_000 } }
        pub fn from_nanos(nanos: u64) -> Duration { Duration { nanos: nanos as u128 } }
        pub fn from_secs_f64(secs: f64) -> Duration { Duration { nanos: (secs * 1e9) as u128 } }
        pub fn as_secs(&self) -> u64 { (self.nanos / 1_000_000_000) as u64 }
        pub fn as_millis(&self) -> u128 { self.nanos / 1_000_000 }
        pub fn as_micros(&self) -> u128 { self.nanos / 1_000 }
        pub fn as_nanos(&self) -> u128 { self.nanos }
        pub fn as_secs_f64(&self) -> f64 { self.nanos as f64 / 1e9 }
        pub fn as_secs_f32(&self) -> f32 { self.nanos as f32 / 1e9 }
        pub fn subsec_millis(&self) -> u32 { ((self.nanos % 1_000_000_000) / 1_000_000) as u32 }
        pub fn subsec_nanos(&self) -> u32 { (self.nanos % 1_000_000_000) as u32 }
    }
    impl crate::ops::Add for Duration {
        type Output = Duration;
        fn add(self, other: Duration) -> Duration { Duration { nanos: self.nanos + other.nanos } }
    }
    impl crate::ops::Sub for Duration {
        type Output = Duration;
        fn sub(self, other: Duration) -> Duration { Duration { nanos: self.nanos - other.nanos } }
    }
    impl crate::fmt::Debug for Duration {
        fn fmt(&self, f: &mut crate::fmt::Formatter) -> crate::fmt::Result { crate::intrinsics::duration_debug(self.nanos, f) }
    }
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    pub struct Instant { nanos: u128 }
    impl Instant {
        pub fn now() -> Instant { Instant { nanos: crate::intrinsics::clock_nanos() } }
        pub fn elapsed(&self) -> Duration { Duration { nanos: crate::intrinsics::clock_nanos() - self.nanos } }
        pub fn duration_since(&self, earlier: Instant) -> Duration { Duration { nanos: self.nanos - earlier.nanos } }
    }
    impl crate::ops::Sub for Instant {
        type Output = Duration;
        fn sub(self, other: Instant) -> Duration { Duration { nanos: self.nanos - other.nanos } }
    }
    pub struct SystemTime { nanos: u128 }
    pub const UNIX_EPOCH: SystemTime = SystemTime { nanos: 0 };
    impl SystemTime {
        pub fn now() -> SystemTime { SystemTime { nanos: crate::intrinsics::wall_nanos() } }
        pub fn duration_since(&self, earlier: SystemTime) -> Result<Duration, ()> { Ok(Duration { nanos: self.nanos - earlier.nanos }) }
    }
}

pub mod io {
    use crate::iter::Lazy;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ErrorKind { NotFound, InvalidInput, InvalidData, UnexpectedEof, Other }
    #[derive(Debug)]
    pub struct Error { kind: ErrorKind, message: String }
    impl Error {
        pub fn new(kind: ErrorKind, message: &str) -> Error { Error { kind: kind, message: message.to_string() } }
        pub fn other(message: &str) -> Error { Error { kind: ErrorKind::Other, message: message.to_string() } }
        pub fn kind(&self) -> ErrorKind { self.kind }
    }
    impl crate::fmt::Display for Error {
        fn fmt(&self, f: &mut crate::fmt::Formatter) -> crate::fmt::Result { f.write_str(&self.message) }
    }
    impl crate::error::Error for Error {}
    pub type Result<T> = core::result::Result<T, Error>;

    pub trait Write {
        fn write_str_out(&mut self, text: &str);
        fn write_fmt<A>(&mut self, text: A) -> Result<()> { self.write_str_out(&crate::intrinsics::arguments_text(text)); Ok(()) }
        fn write_all(&mut self, data: &[u8]) -> Result<()> {
            self.write_str_out(&String::from_utf8_lossy(data));
            Ok(())
        }
        fn write(&mut self, data: &[u8]) -> Result<usize> {
            self.write_str_out(&String::from_utf8_lossy(data));
            Ok(data.len())
        }
        fn flush(&mut self) -> Result<()> { Ok(()) }
    }
    pub trait BufRead {
        fn read_line(&mut self, buf: &mut String) -> Result<usize>;
        fn lines(self) -> Lazy<Result<String>> where Self: Sized { crate::intrinsics::stdin_lines() }
    }
    pub trait Read {
        fn read_to_string(&mut self, buf: &mut String) -> Result<usize>;
    }

    pub struct Stdout { error: bool }
    pub struct Stderr { error: bool }
    pub fn stdout() -> Stdout { Stdout { error: false } }
    pub fn stderr() -> Stderr { Stderr { error: true } }
    impl Stdout { pub fn lock(&self) -> Stdout { Stdout { error: false } } }
    impl Stderr { pub fn lock(&self) -> Stderr { Stderr { error: true } } }
    impl Write for Stdout { fn write_str_out(&mut self, text: &str) { crate::intrinsics::emit(text, false) } }
    impl Write for Stderr { fn write_str_out(&mut self, text: &str) { crate::intrinsics::emit(text, true) } }
    impl Write for Vec<u8> {
        fn write_str_out(&mut self, text: &str) { self.extend_from_slice(text.as_bytes()) }
    }

    pub struct Stdin { marker: () }
    pub fn stdin() -> Stdin { Stdin { marker: () } }
    impl Stdin {
        pub fn lock(&self) -> Stdin { Stdin { marker: () } }
        pub fn read_line(&self, buf: &mut String) -> Result<usize> { Ok(crate::intrinsics::stdin_read_line(buf)) }
        pub fn lines(self) -> Lazy<Result<String>> { crate::intrinsics::stdin_lines() }
    }
    impl BufRead for Stdin {
        fn read_line(&mut self, buf: &mut String) -> Result<usize> { Ok(crate::intrinsics::stdin_read_line(buf)) }
    }
    impl Read for Stdin {
        fn read_to_string(&mut self, buf: &mut String) -> Result<usize> { Ok(crate::intrinsics::stdin_read_all(buf)) }
    }
    pub struct BufReader<R> { inner: R }
    impl<R> BufReader<R> {
        pub fn new(inner: R) -> BufReader<R> { BufReader { inner: inner } }
    }
    impl<R> BufRead for BufReader<R> {
        fn read_line(&mut self, buf: &mut String) -> Result<usize> { Ok(crate::intrinsics::stdin_read_line(buf)) }
    }
    pub struct BufWriter<W> { inner: W }
    impl<W: Write> BufWriter<W> {
        pub fn new(inner: W) -> BufWriter<W> { BufWriter { inner: inner } }
    }
    impl<W: Write> Write for BufWriter<W> {
        fn write_str_out(&mut self, text: &str) { self.inner.write_str_out(text) }
    }
}

pub mod env {
    pub fn args() -> crate::iter::Lazy<String> { crate::intrinsics::env_args() }
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum VarError { NotPresent, NotUnicode(String) }
    impl crate::fmt::Display for VarError {
        fn fmt(&self, f: &mut crate::fmt::Formatter) -> crate::fmt::Result { write!(f, "environment variable not found") }
    }
    pub fn var(key: &str) -> Result<String, VarError> {
        match crate::intrinsics::env_var(key) { Some(value) => Ok(value), None => Err(VarError::NotPresent) }
    }
}

// exit, ExitCode and Termination are built in; the module exists so `use std::process;` resolves.
pub mod process {}
pub mod borrow {
    pub trait Borrow<Borrowed: ?Sized> {
        fn borrow(&self) -> &Borrowed;
    }
    impl<T: ?Sized> Borrow<T> for T { fn borrow(&self) -> &T { self } }
    impl<'a, T: ?Sized> Borrow<T> for &'a T { fn borrow(&self) -> &T { &**self } }
    impl Borrow<str> for String { fn borrow(&self) -> &str { self.as_str() } }
    impl<T> Borrow<[T]> for Vec<T> { fn borrow(&self) -> &[T] { self.as_slice() } }
}

pub mod array {
    use crate::iter::Iterator;

    // Collects exactly N items into an array; callers guarantee the count.
    pub fn collect_exact<I: Iterator, const N: usize>(iter: I) -> [I::Item; N];

    pub fn from_fn<T, const N: usize, F: FnMut(usize) -> T>(f: F) -> [T; N] {
        collect_exact((0..N).map(f))
    }

    impl<T, const N: usize> [T; N] {
        pub fn map<U, F: FnMut(T) -> U>(self, f: F) -> [U; N] { collect_exact(self.into_iter().map(f)) }
        pub fn each_ref(&self) -> [&T; N] { collect_exact(self.iter()) }
        pub fn each_mut(&mut self) -> [&mut T; N] { collect_exact(self.iter_mut()) }
        pub fn as_slice(&self) -> &[T] { self }
        pub fn as_mut_slice(&mut self) -> &mut [T] { self }
    }

    impl<T, const N: usize> crate::intrinsics::Join for [T; N] { type Output = Vec<T>; }

    impl<T> [T] {
        pub fn chunks_mut(&mut self, size: usize) -> crate::iter::Lazy<&mut [T]>;
        pub fn chunks_exact_mut(&mut self, size: usize) -> crate::iter::Lazy<&mut [T]>;
        pub fn rchunks_mut(&mut self, size: usize) -> crate::iter::Lazy<&mut [T]>;
        pub fn select_nth_unstable(&mut self, index: usize) -> (&mut [T], &mut T, &mut [T]) where T: Ord {
            self.sort_unstable();
            let (left, rest) = self.split_at_mut(index);
            let (middle, right) = rest.split_at_mut(1);
            (left, &mut middle[0], right)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TryFromSliceError(());
    impl crate::fmt::Display for TryFromSliceError {
        fn fmt(&self, f: &mut crate::fmt::Formatter) -> crate::fmt::Result { write!(f, "could not convert slice to array") }
    }
    impl crate::error::Error for TryFromSliceError {}

    impl<T, const N: usize> TryFrom<Vec<T>> for [T; N] {
        type Error = Vec<T>;
        fn try_from(value: Vec<T>) -> Result<[T; N], Vec<T>> {
            if value.len() == N { Ok(collect_exact(value.into_iter())) } else { Err(value) }
        }
    }
    impl<T: Copy, const N: usize> TryFrom<&[T]> for [T; N] {
        type Error = TryFromSliceError;
        fn try_from(value: &[T]) -> Result<[T; N], TryFromSliceError> {
            if value.len() == N { Ok(collect_exact(value.iter().copied())) } else { Err(TryFromSliceError(())) }
        }
    }
}

pub mod i8 {
    pub const MIN: i8 = i8::MIN;
    pub const MAX: i8 = i8::MAX;
}
pub mod i16 {
    pub const MIN: i16 = i16::MIN;
    pub const MAX: i16 = i16::MAX;
}
pub mod i32 {
    pub const MIN: i32 = i32::MIN;
    pub const MAX: i32 = i32::MAX;
}
pub mod i64 {
    pub const MIN: i64 = i64::MIN;
    pub const MAX: i64 = i64::MAX;
}
pub mod i128 {
    pub const MIN: i128 = i128::MIN;
    pub const MAX: i128 = i128::MAX;
}
pub mod isize {
    pub const MIN: isize = isize::MIN;
    pub const MAX: isize = isize::MAX;
}
pub mod u8 {
    pub const MIN: u8 = u8::MIN;
    pub const MAX: u8 = u8::MAX;
}
pub mod u16 {
    pub const MIN: u16 = u16::MIN;
    pub const MAX: u16 = u16::MAX;
}
pub mod u32 {
    pub const MIN: u32 = u32::MIN;
    pub const MAX: u32 = u32::MAX;
}
pub mod u64 {
    pub const MIN: u64 = u64::MIN;
    pub const MAX: u64 = u64::MAX;
}
pub mod u128 {
    pub const MIN: u128 = u128::MIN;
    pub const MAX: u128 = u128::MAX;
}
pub mod usize {
    pub const MIN: usize = usize::MIN;
    pub const MAX: usize = usize::MAX;
}

pub mod num_nonzero {
    use crate::fmt;
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonZero<T>(T);
    impl<T: Copy + PartialEq + Default> NonZero<T> {
        pub fn new(n: T) -> Option<NonZero<T>> { if n == T::default() { None } else { Some(NonZero(n)) } }
        pub fn get(self) -> T { self.0 }
    }
    impl<T: fmt::Debug> fmt::Debug for NonZero<T> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::Debug::fmt(&self.0, f) }
    }
    impl<T: fmt::Display> fmt::Display for NonZero<T> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::Display::fmt(&self.0, f) }
    }
}

pub mod bool {
    impl bool {
        pub fn then<T, F: FnOnce() -> T>(self, f: F) -> Option<T> { if self { Some(f()) } else { None } }
        pub fn then_some<T>(self, t: T) -> Option<T> { if self { Some(t) } else { None } }
    }
}

pub mod ascii_bytes {
    impl u8 {
        pub fn is_ascii(&self) -> bool { *self < 128 }
        pub fn is_ascii_digit(&self) -> bool { (*self as char).is_ascii_digit() }
        pub fn is_ascii_hexdigit(&self) -> bool { (*self as char).is_ascii_hexdigit() }
        pub fn is_ascii_alphabetic(&self) -> bool { (*self as char).is_ascii_alphabetic() }
        pub fn is_ascii_alphanumeric(&self) -> bool { (*self as char).is_ascii_alphanumeric() }
        pub fn is_ascii_uppercase(&self) -> bool { (*self as char).is_ascii_uppercase() }
        pub fn is_ascii_lowercase(&self) -> bool { (*self as char).is_ascii_lowercase() }
        pub fn is_ascii_punctuation(&self) -> bool { (*self as char).is_ascii_punctuation() }
        pub fn is_ascii_whitespace(&self) -> bool { (*self as char).is_ascii_whitespace() }
        pub fn is_ascii_graphic(&self) -> bool { (*self as char).is_ascii_graphic() }
        pub fn is_ascii_control(&self) -> bool { (*self as char).is_ascii_control() }
        pub fn to_ascii_uppercase(&self) -> u8 { if *self >= b'a' && *self <= b'z' { *self - 32 } else { *self } }
        pub fn to_ascii_lowercase(&self) -> u8 { if *self >= b'A' && *self <= b'Z' { *self + 32 } else { *self } }
        pub fn make_ascii_uppercase(&mut self) { *self = self.to_ascii_uppercase(); }
        pub fn make_ascii_lowercase(&mut self) { *self = self.to_ascii_lowercase(); }
        pub fn eq_ignore_ascii_case(&self, other: &u8) -> bool { self.to_ascii_lowercase() == other.to_ascii_lowercase() }
    }
}
