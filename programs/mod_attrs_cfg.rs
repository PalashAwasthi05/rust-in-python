// #[derive] on items in modules, #[cfg(test)] modules, and #[allow] attrs.
#![allow(dead_code)]
#![allow(unused_imports)]

mod inventory {
    #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct Item {
        pub name: String,
        pub qty: u32,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
    pub enum Kind {
        #[default]
        Tool,
        Food,
    }

    #[allow(unused_variables)]
    pub fn make(name: &str, qty: u32) -> Item {
        let unused = 5;
        Item { name: name.into(), qty }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn it_works() {
            assert_eq!(make("a", 1).qty, 1);
        }

        fn this_does_not_compile() -> NotAType {
            undefined_function()
        }
    }

    #[cfg(test)]
    pub fn only_in_tests() -> u32 {
        missing()
    }

    #[inline]
    #[must_use]
    pub fn total(items: &[Item]) -> u32 {
        items.iter().map(|i| i.qty).sum()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn whatever() {
        panic!("never runs");
    }
}

#[allow(non_snake_case)]
fn BadName() -> &'static str {
    "allowed"
}

#[allow(non_camel_case_types)]
struct lower_case(i32);

use inventory::{Item, Kind};
use std::collections::{BTreeSet, HashSet};

fn main() {
    let a = inventory::make("hammer", 2);
    let b = inventory::make("apple", 5);
    let mut items = vec![a.clone(), b.clone(), inventory::make("apple", 1)];
    items.sort();
    println!("{:?}", items);
    println!("{}", inventory::total(&items));
    let set: HashSet<Item> = items.iter().cloned().collect();
    println!("{}", set.len());
    let kinds: BTreeSet<Kind> = [Kind::Food, Kind::Tool, Kind::Food].into_iter().collect();
    println!("{:?} {:?}", kinds, Kind::default());
    println!("{} {}", a == a.clone(), a < b);
    println!("{} {}", BadName(), lower_case(3).0);
    #[allow(unused_mut)]
    let mut z = 1;
    println!("{}", z);
}
