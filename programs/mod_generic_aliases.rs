// Type aliases with generics, inside modules and used across them.
mod types {
    use std::collections::HashMap;

    pub type Pair<T> = (T, T);
    pub type Table<V> = HashMap<String, V>;
    pub type Res<T, E = String> = Result<T, E>;
    pub type Callback = fn(i32) -> i32;
    pub type Grid<const N: usize> = [[u8; N]; N];

    pub struct Wrapper<T> {
        pub inner: T,
    }

    pub type IntWrapper = Wrapper<i32>;

    impl<T: Clone> Wrapper<T> {
        pub fn twice(&self) -> Pair<T> {
            (self.inner.clone(), self.inner.clone())
        }
    }
}

use types::{Callback, Pair, Res, Table};

fn swap<T>(p: Pair<T>) -> Pair<T> {
    (p.1, p.0)
}

fn parse(s: &str) -> Res<i32> {
    s.parse::<i32>().map_err(|e| format!("bad {:?}: {}", s, e))
}

fn parse_u8(s: &str) -> Res<u8, ()> {
    s.parse().map_err(|_| ())
}

fn apply(f: Callback, x: i32) -> i32 {
    f(x)
}

fn triple(x: i32) -> i32 {
    x * 3
}

fn main() {
    println!("{:?}", swap((1, 2)));
    println!("{:?}", swap(("a", "b")));
    let mut t: Table<u32> = Table::new();
    t.insert("x".to_string(), 5);
    println!("{:?}", t.get("x"));
    println!("{:?} {:?}", parse("42"), parse("4x"));
    println!("{:?} {:?}", parse_u8("200"), parse_u8("300"));
    println!("{}", apply(triple, 7));
    let w = types::IntWrapper { inner: 9 };
    println!("{:?}", w.twice());
    let w2 = types::IntWrapper { inner: 2 };
    println!("{}", w2.inner);
    let g: types::Grid<3> = [[1; 3]; 3];
    println!("{}", g.iter().map(|r| r.iter().map(|&x| x as u32).sum::<u32>()).sum::<u32>());
}
