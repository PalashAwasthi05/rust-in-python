// The parts of Rust's standard library this interpreter provides, written in Rust.
// Body-less functions are intrinsics implemented in rust_in_python.py.

pub mod prelude {
    pub use crate::iter::Iterator;
    pub use crate::iter::IntoIterator;
    pub use crate::cmp::Ordering;
    pub use crate::mem::drop;
}

pub mod intrinsics {
    pub fn vec_last<T>(vector: &Vec<T>) -> Option<&T>;
    pub fn iter_max<I: crate::iter::Iterator>(iter: I, last: bool) -> Option<I::Item>;
    pub fn iter_collect<I: crate::iter::Iterator>(iter: I) -> Vec<I::Item>;
    pub fn compare<T>(left: &T, right: &T) -> i32;
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
    #[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
    pub enum Ordering { Less, Equal, Greater }
}

pub mod vec {
    impl<T> Vec<T> {
        pub fn last(&self) -> Option<&T> { crate::intrinsics::vec_last(self) }
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
}

pub mod clone {
}

pub mod default {
}

pub mod marker {
}

pub mod ops {
}
