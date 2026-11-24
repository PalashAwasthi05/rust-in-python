// Chapter 13.1: closures that capture their environment, Fn traits, and a memoizing cache.
use std::collections::HashMap;
use std::thread;

#[derive(Debug, PartialEq, Copy, Clone)]
enum ShirtColor {
    Red,
    Blue,
}

struct Inventory {
    shirts: Vec<ShirtColor>,
}

impl Inventory {
    fn giveaway(&self, user_preference: Option<ShirtColor>) -> ShirtColor {
        user_preference.unwrap_or_else(|| self.most_stocked())
    }

    fn most_stocked(&self) -> ShirtColor {
        let mut num_red = 0;
        let mut num_blue = 0;
        for color in &self.shirts {
            match color {
                ShirtColor::Red => num_red += 1,
                ShirtColor::Blue => num_blue += 1,
            }
        }
        if num_red > num_blue {
            ShirtColor::Red
        } else {
            ShirtColor::Blue
        }
    }
}

#[derive(Debug)]
struct Rectangle {
    width: u32,
    height: u32,
}

struct Cacher<T>
where
    T: Fn(u64) -> u64,
{
    calculation: T,
    values: HashMap<u64, u64>,
    calls: u32,
}

impl<T> Cacher<T>
where
    T: Fn(u64) -> u64,
{
    fn new(calculation: T) -> Cacher<T> {
        Cacher {
            calculation,
            values: HashMap::new(),
            calls: 0,
        }
    }

    fn value(&mut self, arg: u64) -> u64 {
        if let Some(&v) = self.values.get(&arg) {
            return v;
        }
        self.calls += 1;
        let v = (self.calculation)(arg);
        self.values.insert(arg, v);
        v
    }
}

fn apply<F: FnOnce() -> String>(f: F) -> String {
    f()
}

fn apply_twice<F>(mut f: F)
where
    F: FnMut(),
{
    f();
    f();
}

fn compose<A, B, C>(f: impl Fn(A) -> B, g: impl Fn(B) -> C) -> impl Fn(A) -> C {
    move |x| g(f(x))
}

fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}

fn make_counter() -> impl FnMut() -> u32 {
    let mut count = 0;
    move || {
        count += 1;
        count
    }
}

fn main() {
    let store = Inventory {
        shirts: vec![ShirtColor::Blue, ShirtColor::Red, ShirtColor::Blue],
    };
    let user_pref1 = Some(ShirtColor::Red);
    let giveaway1 = store.giveaway(user_pref1);
    println!("The user with preference {:?} gets {:?}", user_pref1, giveaway1);
    let user_pref2 = None;
    let giveaway2 = store.giveaway(user_pref2);
    println!("The user with preference {:?} gets {:?}", user_pref2, giveaway2);

    let expensive_closure = |num: u32| -> u32 { num * 2 };
    println!("expensive_closure(21) = {}", expensive_closure(21));
    let add_one_v2 = |x: u32| -> u32 { x + 1 };
    let add_one_v3 = |x| x + 1;
    let add_one_v4 = |x| x + 1;
    println!("{} {} {}", add_one_v2(1), add_one_v3(2u8), add_one_v4(3i64));

    let example_closure = |x| x;
    let s = example_closure(String::from("hello"));
    println!("{s}");

    let list = vec![1, 2, 3];
    println!("Before defining closure: {list:?}");
    let only_borrows = || println!("From closure: {list:?}");
    println!("Before calling closure: {list:?}");
    only_borrows();
    println!("After calling closure: {list:?}");

    let mut list = vec![1, 2, 3];
    println!("Before defining closure: {list:?}");
    let mut borrows_mutably = || list.push(7);
    borrows_mutably();
    println!("After calling closure: {list:?}");

    let list = vec![1, 2, 3];
    println!("Before defining closure: {list:?}");
    thread::spawn(move || println!("From thread: {list:?}"))
        .join()
        .unwrap();

    let mut list = [
        Rectangle { width: 10, height: 1 },
        Rectangle { width: 3, height: 5 },
        Rectangle { width: 7, height: 12 },
    ];
    let mut num_sort_operations = 0;
    list.sort_by_key(|r| {
        num_sort_operations += 1;
        r.width
    });
    println!("{list:#?}, sorted with key calls > 0: {}", num_sort_operations > 0);

    let mut c = Cacher::new(|n| {
        println!("calculating slowly for {n}...");
        n * n + 1
    });
    for arg in [3, 4, 3, 3, 4, 10] {
        println!("value({arg}) = {}", c.value(arg));
    }
    println!("calls = {}", c.calls);

    let name = String::from("Ferris");
    let greeting = apply(move || format!("hello, {name}"));
    println!("{greeting}");
    let mut total = 0;
    apply_twice(|| total += 5);
    println!("total = {total}");

    let parse_then_double = compose(|s: &str| s.len(), |n: usize| n * 2);
    println!("{}", parse_then_double("four"));
    let add5 = make_adder(5);
    let adders: Vec<Box<dyn Fn(i32) -> i32>> = vec![Box::new(add5), Box::new(make_adder(-1)), Box::new(|x| x * x)];
    println!("{:?}", adders.iter().map(|f| f(7)).collect::<Vec<_>>());
    let mut counter = make_counter();
    let mut other = make_counter();
    println!("{} {} {} | {}", counter(), counter(), counter(), other());

    let words = vec!["apple", "bob", "kayak", "rust"];
    let is_palindrome = |w: &&str| w.chars().eq(w.chars().rev());
    let palindromes: Vec<_> = words.iter().filter(|w| is_palindrome(*w)).collect();
    println!("{:?}", palindromes);
    let threshold = 4;
    let long_words: Vec<String> = words
        .iter()
        .filter(|w| w.len() >= threshold)
        .map(|w| w.to_uppercase())
        .collect();
    println!("{:?}", long_words);
    let fs: Vec<fn(i32) -> i32> = vec![|x| x + 1, |x| x * 10];
    println!("{:?}", fs.iter().map(|f| f(3)).collect::<Vec<_>>());
}
