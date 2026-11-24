use std::fmt::Debug;

// explicit annotation: the result lives as long as both inputs
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

// only the first input is tied to the output
fn first_of<'a, 'b>(x: &'a str, _y: &'b str) -> &'a str { x }

// elided: one input reference, so the output borrows from it
fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or("")
}

fn add_one<'a>(x: &'a mut i32) { *x += 1; }

#[derive(Debug)]
struct Borrowed<'a>(&'a i32);

#[derive(Debug)]
struct NamedBorrowed<'a> { x: &'a i32, y: &'a i32 }

#[derive(Debug)]
enum Either<'a> { Num(i32), Ref(&'a i32) }

struct Highlighter<'t> { text: &'t str }

impl<'t> Highlighter<'t> {
    fn find(&self, needle: &str) -> Option<&'t str> {
        let start = self.text.find(needle)?;
        Some(&self.text[start..start + needle.len()])
    }
    fn words_longer_than(&self, n: usize) -> Vec<&'t str> {
        self.text.split_whitespace().filter(|w| w.len() > n).collect()
    }
}

trait Describe<'a> { fn describe(&self) -> &'a str; }

impl<'a> Describe<'a> for Borrowed<'a> {
    fn describe(&self) -> &'a str { if *self.0 > 5 { "big" } else { "small" } }
}

// bounds: T must outlive 'a because we hold a reference to it
#[derive(Debug)]
struct Ref<'a, T: 'a>(&'a T);

fn print_ref<'a, T>(t: &'a T) where T: Debug + 'a {
    println!("print_ref: t is {:?}", t);
}

// coercion: a longer lifetime is shortened to fit
fn multiply<'a>(first: &'a i32, second: &'a i32) -> i32 { first * second }

fn choose_first<'a: 'b, 'b>(first: &'a i32, _: &'b i32) -> &'b i32 { first }

static NUM: i32 = 18;

fn coerce_static<'a>(_: &'a i32) -> &'a i32 { &NUM }

fn main() {
    let s1 = String::from("long string is long");
    let result;
    {
        let s2 = String::from("xyz");
        result = longest(s1.as_str(), s2.as_str()).to_string();
    }
    println!("the longest string is {}", result);
    let kept;
    {
        let temp = String::from("temporary");
        kept = first_of(&s1, &temp);
    }
    println!("first_of outlives temp: {}", kept);
    println!("first word: {:?}", first_word("  hello world"));

    let mut n = 3;
    add_one(&mut n);
    println!("after add_one: {}", n);

    let x = 18;
    let y = 15;
    let single = Borrowed(&x);
    let double = NamedBorrowed { x: &x, y: &y };
    let reference = Either::Ref(&x);
    let number = Either::Num(y);
    println!("x is borrowed in {:?}", single);
    println!("x and y are borrowed in {:?}", double);
    println!("x is borrowed in {:?}", reference);
    println!("y is *not* borrowed in {:?}", number);
    println!("Borrowed describes itself as {}", single.describe());

    let text = String::from("the quick brown fox jumps over the lazy dog");
    let h = Highlighter { text: &text };
    let found = h.find("brown");
    let long_words = h.words_longer_than(4);
    drop(h);
    println!("found {:?} and long words {:?} after highlighter dropped", found, long_words);

    let r = Ref(&7);
    print_ref(&r);
    let first = 2;
    {
        let second = 3;
        println!("the product is {}", multiply(&first, &second));
        println!("{} is the first", choose_first(&first, &second));
    }
    {
        let static_string: &'static str = "I'm in read-only memory";
        println!("static_string: {}", static_string);
    }
    let lifetime_num = 9;
    let coerced = coerce_static(&lifetime_num);
    println!("coerced_static: {}", coerced);
    println!("NUM: {} stays accessible!", NUM);
}
