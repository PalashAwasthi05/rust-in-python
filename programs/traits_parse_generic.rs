// A generic trait implemented several times for one type, selected by the requested result type.
use std::collections::HashMap;
use std::str::FromStr;

trait Parse<T> {
    fn parse_one(&self, s: &str) -> Option<T>;
}

struct Parser;

impl Parse<i32> for Parser {
    fn parse_one(&self, s: &str) -> Option<i32> {
        s.trim().parse().ok()
    }
}

impl Parse<bool> for Parser {
    fn parse_one(&self, s: &str) -> Option<bool> {
        match s.trim() {
            "yes" | "true" => Some(true),
            "no" | "false" => Some(false),
            _ => None,
        }
    }
}

fn parse_all<T>(p: &impl Parse<T>, items: &[&str]) -> Vec<Option<T>> {
    items.iter().map(|s| p.parse_one(s)).collect()
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Op {
    Push(i64),
    Add,
    Mul,
    Dup,
}

impl FromStr for Op {
    type Err = String;
    fn from_str(s: &str) -> Result<Op, String> {
        Ok(match s {
            "add" => Op::Add,
            "mul" => Op::Mul,
            "dup" => Op::Dup,
            _ => Op::Push(s.parse().map_err(|_| format!("bad op {:?}", s))?),
        })
    }
}

fn parse_list<T: FromStr>(line: &str) -> Result<Vec<T>, T::Err> {
    line.split_whitespace().map(|w| w.parse::<T>()).collect()
}

trait Lookup<K> {
    type Value;
    fn lookup(&self, k: K) -> Option<&Self::Value>;
}

struct Phonebook {
    by_name: HashMap<String, u32>,
    names: Vec<String>,
}

impl Lookup<&str> for Phonebook {
    type Value = u32;
    fn lookup(&self, k: &str) -> Option<&u32> {
        self.by_name.get(k)
    }
}

impl Lookup<usize> for Phonebook {
    type Value = String;
    fn lookup(&self, k: usize) -> Option<&String> {
        self.names.get(k)
    }
}

fn main() {
    let n: Option<i32> = Parser.parse_one(" 42 ");
    let b: Option<bool> = Parser.parse_one("yes");
    println!("{:?} {:?}", n, b);
    let nums: Vec<Option<i32>> = parse_all(&Parser, &["1", "x", "-7"]);
    let flags = parse_all::<bool>(&Parser, &["no", "maybe", "true"]);
    println!("{:?} {:?}", nums, flags);
    let total: i32 = nums.iter().flatten().sum();
    println!("{}", total);
    let prog: Vec<Op> = parse_list("3 dup mul 4 add").unwrap();
    println!("{:?}", prog);
    let mut stack = Vec::new();
    for op in &prog {
        match *op {
            Op::Push(v) => stack.push(v),
            Op::Dup => {
                let top = *stack.last().unwrap();
                stack.push(top);
            }
            Op::Add | Op::Mul => {
                let (b, a) = (stack.pop().unwrap(), stack.pop().unwrap());
                stack.push(if *op == Op::Add { a + b } else { a * b });
            }
        }
    }
    println!("{:?}", stack);
    println!("{:?}", parse_list::<Op>("1 sub"));
    println!("{:?}", parse_list::<u8>("1 2 300"));
    println!("{:?}", parse_list::<f64>("1.5 -2e3"));
    let mut book = Phonebook { by_name: HashMap::new(), names: vec![] };
    for (i, name) in ["ann", "bob"].iter().enumerate() {
        book.by_name.insert(name.to_string(), 5550 + i as u32);
        book.names.push(name.to_string());
    }
    println!("{:?} {:?}", book.lookup("bob"), book.lookup("zed"));
    println!("{:?} {:?}", book.lookup(0usize), book.lookup(9usize));
}
