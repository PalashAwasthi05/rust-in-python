// Custom iterators borrowing from a struct, generic over the element type.
use std::fmt::Debug;

struct Iter<'a, T> {
    slice: &'a [T],
    pos: usize,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        if self.pos < self.slice.len() {
            let item = &self.slice[self.pos];
            self.pos += 1;
            Some(item)
        } else {
            None
        }
    }
}

struct Windows<'a, T> {
    slice: &'a [T],
    size: usize,
    start: usize,
}

impl<'a, T> Iterator for Windows<'a, T> {
    type Item = &'a [T];
    fn next(&mut self) -> Option<Self::Item> {
        if self.start + self.size > self.slice.len() {
            return None;
        }
        let window = &self.slice[self.start..self.start + self.size];
        self.start += 1;
        Some(window)
    }
}

struct Stack<T> {
    items: Vec<T>,
}

impl<T: Debug> Stack<T> {
    fn new() -> Self {
        Stack { items: Vec::new() }
    }
    fn push(&mut self, item: T) {
        self.items.push(item);
    }
    fn iter(&self) -> Iter<'_, T> {
        Iter { slice: &self.items, pos: 0 }
    }
    fn windows(&self, size: usize) -> Windows<'_, T> {
        Windows { slice: &self.items, size, start: 0 }
    }
}

struct Pairs<'a> {
    words: &'a [&'a str],
    i: usize,
    j: usize,
}

impl<'a> Iterator for Pairs<'a> {
    type Item = (&'a str, &'a str);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.i >= self.words.len() {
                return None;
            }
            self.j += 1;
            if self.j < self.words.len() {
                return Some((self.words[self.i], self.words[self.j]));
            }
            self.i += 1;
            self.j = self.i;
        }
    }
}

fn longest<'a>(words: &'a [String]) -> Option<&'a String> {
    let it = Iter { slice: words, pos: 0 };
    it.fold(None, |best: Option<&String>, w| match best {
        Some(b) if b.len() >= w.len() => Some(b),
        _ => Some(w),
    })
}

fn main() {
    let mut stack = Stack::new();
    for n in [4, 8, 15, 16, 23, 42] {
        stack.push(n);
    }
    let doubled: Vec<i32> = stack.iter().map(|x| x * 2).collect();
    println!("doubled {:?}", doubled);
    let refs: Vec<&i32> = stack.iter().filter(|x| **x % 2 == 0).collect();
    println!("even refs {:?}", refs);
    let total: i32 = stack.iter().sum();
    println!("sum {}", total);
    let copied: Vec<i32> = stack.iter().copied().filter(|x| x % 3 != 0).collect();
    println!("copied {:?}", copied);
    let max = stack.iter().max();
    println!("max {:?}", max);

    for w in stack.windows(3) {
        println!("window {:?} avg {:.2}", w, w.iter().sum::<i32>() as f64 / w.len() as f64);
    }
    let increasing = stack.windows(2).all(|w| w[0] < w[1]);
    println!("increasing: {}", increasing);

    let mut names: Stack<String> = Stack::new();
    names.push("ada".to_string());
    names.push("grace".to_string());
    names.push("barbara".to_string());
    let joined: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    println!("{}", joined.join("/"));
    println!("longest {:?}", longest(&names.items));
    let cloned: Vec<String> = names.iter().cloned().collect();
    println!("cloned {:?}", cloned);

    let words = ["a", "b", "c", "d"];
    let pairs: Vec<String> = Pairs { words: &words, i: 0, j: 0 }.map(|(x, y)| format!("{}{}", x, y)).collect();
    println!("pairs {:?}", pairs);
    println!("pair count {}", Pairs { words: &words[..3], i: 0, j: 0 }.count());
}
