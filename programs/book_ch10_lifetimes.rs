// Chapter 10.3: validating references with lifetimes.
use std::fmt::Display;

fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}

fn first<'a>(x: &'a str, _y: &str) -> &'a str {
    x
}

#[derive(Debug)]
struct ImportantExcerpt<'a> {
    part: &'a str,
}

impl<'a> ImportantExcerpt<'a> {
    fn level(&self) -> i32 {
        3
    }

    fn announce_and_return_part(&self, announcement: &str) -> &str {
        println!("Attention please: {announcement}");
        self.part
    }

    fn words(&self) -> Vec<&'a str> {
        self.part.split_whitespace().collect()
    }
}

fn longest_with_an_announcement<'a, T>(x: &'a str, y: &'a str, ann: T) -> &'a str
where
    T: Display,
{
    println!("Announcement! {ann}");
    if x.len() > y.len() {
        x
    } else {
        y
    }
}

struct Parser<'s> {
    source: &'s str,
    pos: usize,
}

impl<'s> Parser<'s> {
    fn new(source: &'s str) -> Self {
        Parser { source, pos: 0 }
    }

    fn next_token(&mut self) -> Option<&'s str> {
        let rest = &self.source[self.pos..];
        let trimmed = rest.trim_start();
        if trimmed.is_empty() {
            return None;
        }
        let start = self.pos + (rest.len() - trimmed.len());
        let len = trimmed.find(' ').unwrap_or(trimmed.len());
        self.pos = start + len;
        Some(&self.source[start..start + len])
    }
}

fn longest_in<'a>(items: &[&'a str]) -> Option<&'a str> {
    let mut best: Option<&'a str> = None;
    for &item in items {
        best = match best {
            Some(b) if b.len() >= item.len() => Some(b),
            _ => Some(item),
        };
    }
    best
}

static GREETING: &str = "I have a static lifetime.";

fn pick<'a, 'b: 'a>(flag: bool, a: &'a str, b: &'b str) -> &'a str {
    if flag { a } else { b }
}

fn main() {
    let string1 = String::from("abcd");
    let string2 = "xyz";
    let result = longest(string1.as_str(), string2);
    println!("The longest string is {result}");

    let string1 = String::from("long string is long");
    {
        let string2 = String::from("xyz");
        let result = longest(string1.as_str(), string2.as_str());
        println!("The longest string is {result}");
    }
    let result;
    {
        let string2 = "a literal lives forever";
        result = longest(string1.as_str(), string2);
    }
    println!("The longest string is {result}");
    println!("first: {}", first("alpha", "beta"));

    let novel = String::from("Call me Ishmael. Some years ago...");
    let first_sentence = novel.split('.').next().unwrap();
    let i = ImportantExcerpt { part: first_sentence };
    println!("{:?} level {}", i, i.level());
    println!("returned: {}", i.announce_and_return_part("hear ye"));
    println!("words: {:?}", i.words());

    let s: &'static str = "I have a static lifetime.";
    println!("{s} == GREETING? {}", s == GREETING);

    println!("{}", longest_with_an_announcement("short", "much longer", 42));
    println!("{}", longest_with_an_announcement("tiny", "", "done"));

    let text = String::from("  let   x =  5 ;");
    let mut parser = Parser::new(&text);
    let mut tokens = Vec::new();
    while let Some(tok) = parser.next_token() {
        tokens.push(tok);
    }
    println!("{:?}", tokens);

    let names = ["Ann", "Bartholomew", "Cy", "Dorothea"];
    println!("{:?}", longest_in(&names));
    println!("{:?}", longest_in(&[]));
    let owned = String::from("owned");
    println!("{} {}", pick(true, &owned, GREETING), pick(false, &owned, GREETING));
}
