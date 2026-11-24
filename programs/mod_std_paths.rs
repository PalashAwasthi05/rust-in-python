// std:: vs ::std:: vs core:: paths, and use std::fmt::{self, Display}.
use ::std::collections::BTreeSet;
use core::cmp::Reverse;
use std::fmt::{self, Debug, Display};

struct Celsius(f64);

impl Display for Celsius {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1}C", self.0)
    }
}

impl Debug for Celsius {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Celsius").field(&self.0).finish()
    }
}

fn show<T: Display + Debug>(t: T) -> String {
    format!("{} / {:?}", t, t)
}

fn main() {
    println!("{}", show(Celsius(21.456)));
    let mut set = BTreeSet::new();
    set.insert(3);
    set.insert(1);
    println!("{:?}", set);
    let mut v = vec![4, 1, 3];
    v.sort_by_key(|&x| Reverse(x));
    println!("{:?}", v);
    println!("{}", ::std::cmp::max(1, 2));
    println!("{}", core::cmp::min(1, 2));
    println!("{}", std::i32::MAX);
    println!("{}", core::u8::MAX);
    println!("{}", i64::MAX);
    let o: core::option::Option<i32> = core::option::Option::Some(5);
    println!("{:?}", o);
    let s: ::std::string::String = ::std::string::String::from("abc");
    println!("{}", s.len());
    let r: std::result::Result<i32, ()> = std::result::Result::Ok(1);
    println!("{:?}", r);
    println!("{}", core::mem::size_of::<u32>());
    let mut a = 1;
    let mut b = 2;
    ::core::mem::swap(&mut a, &mut b);
    println!("{} {}", a, b);
    let words: ::std::vec::Vec<&str> = "a b c".split(' ').collect();
    println!("{:?}", words);
}
