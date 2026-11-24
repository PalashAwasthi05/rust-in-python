trait Container {
    type Item;
    const KIND: &'static str;
    const CAPACITY: usize = 4;
    fn items(&self) -> Vec<Self::Item>;
    fn first(&self) -> Option<Self::Item> {
        self.items().into_iter().next()
    }
    fn is_full(&self) -> bool {
        self.items().len() >= Self::CAPACITY
    }
}

struct Stack {
    data: Vec<i32>,
}

struct Letters {
    text: String,
}

impl Container for Stack {
    type Item = i32;
    const KIND: &'static str = "stack";
    fn items(&self) -> Vec<i32> {
        self.data.iter().rev().cloned().collect()
    }
}

impl Container for Letters {
    type Item = char;
    const KIND: &'static str = "letters";
    const CAPACITY: usize = 10;
    fn items(&self) -> Vec<char> {
        self.text.chars().filter(|c| c.is_alphabetic()).collect()
    }
}

fn summary<C: Container>(c: &C) -> String
where
    C::Item: std::fmt::Debug,
{
    format!("{} (cap {}): {:?} first={:?} full={}", C::KIND, C::CAPACITY, c.items(), c.first(), c.is_full())
}

trait Graph {
    type Node: Copy + std::fmt::Debug + PartialEq;
    fn neighbors(&self, n: Self::Node) -> Vec<Self::Node>;
}

struct Ring(u8);

impl Graph for Ring {
    type Node = u8;
    fn neighbors(&self, n: u8) -> Vec<u8> {
        vec![(n + 1) % self.0, (n + self.0 - 1) % self.0]
    }
}

fn walk<G: Graph>(g: &G, start: G::Node, steps: usize) -> Vec<G::Node> {
    let mut path = vec![start];
    let mut cur = start;
    for _ in 0..steps {
        cur = g.neighbors(cur)[0];
        path.push(cur);
    }
    path
}

trait Unit {
    const FACTOR: f64;
    const SYMBOL: char;
}
struct Km;
struct Mile;
impl Unit for Km {
    const FACTOR: f64 = 1000.0;
    const SYMBOL: char = 'k';
}
impl Unit for Mile {
    const FACTOR: f64 = 1609.344;
    const SYMBOL: char = 'm';
}

fn to_meters<U: Unit>(x: f64) -> String {
    format!("{}{} = {:.1}m", x, U::SYMBOL, x * U::FACTOR)
}

fn main() {
    let s = Stack { data: vec![1, 2, 3, 4, 5] };
    let l = Letters { text: "a1b2c".into() };
    println!("{}", summary(&s));
    println!("{}", summary(&l));
    println!("{}", <Stack as Container>::KIND);
    println!("{:?}", walk(&Ring(5), 3, 6));
    println!("{}", to_meters::<Km>(2.5));
    println!("{}", to_meters::<Mile>(3.0));
    let e = Stack { data: vec![] };
    println!("{:?} {}", e.first(), e.is_full());
}
