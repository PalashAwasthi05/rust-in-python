use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

#[derive(Debug, Eq)]
struct Tag(String, u32);
impl PartialEq for Tag {
    fn eq(&self, o: &Tag) -> bool { self.0.to_lowercase() == o.0.to_lowercase() }
}
impl Hash for Tag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_lowercase().hash(state);
    }
}
#[derive(Hash, PartialEq, Eq, Debug)]
struct P { x: i32, y: (u8, char) }

fn h<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

fn main() {
    let mut s = HashSet::new();
    println!("{}", s.insert(Tag("A".into(), 1)));
    println!("{}", s.insert(Tag("a".into(), 2)));
    println!("{}", s.len());
    println!("{}", h(&Tag("Q".into(), 1)) == h(&Tag("q".into(), 5)));
    println!("{}", h(&P { x: 1, y: (2, 'c') }) == h(&P { x: 1, y: (2, 'c') }));
    println!("{}", h(&P { x: 1, y: (2, 'c') }) == h(&P { x: 1, y: (3, 'c') }));
    println!("{}", h(&"abc") == h(&String::from("abc")));
    println!("{}", h(&5u8) == h(&6u8));
}
