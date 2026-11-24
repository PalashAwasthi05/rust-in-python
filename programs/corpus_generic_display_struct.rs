use std::fmt::{self, Display};

#[derive(Debug, Clone, PartialEq)]
struct Pair<T> {
    a: T,
    b: T,
}

impl<T> Pair<T> {
    fn new(a: T, b: T) -> Self { Pair { a, b } }
    fn swap(self) -> Pair<T> { Pair { a: self.b, b: self.a } }
}

impl<T: Display + PartialOrd> Pair<T> {
    fn largest(&self) -> &T {
        if self.a >= self.b { &self.a } else { &self.b }
    }
    fn announce(&self) -> String {
        format!("the larger of {} and {} is {}", self.a, self.b, self.largest())
    }
}

impl<T: Display> Display for Pair<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}, {}>", self.a, self.b)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Celsius(f64);

impl Display for Celsius {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.1}°C", self.0)
    }
}

struct Bounded<T> { items: Vec<T>, cap: usize }

impl<T: PartialOrd + Display + Clone> Bounded<T> {
    fn push(&mut self, x: T) -> Result<(), String> {
        if self.items.len() == self.cap {
            return Err(format!("full, dropped {}", x));
        }
        self.items.push(x);
        Ok(())
    }
    fn max(&self) -> Option<T> {
        self.items.iter().fold(None, |best: Option<&T>, x| match best {
            Some(b) if b >= x => Some(b),
            _ => Some(x),
        }).cloned()
    }
    fn render(&self) -> String {
        self.items.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" | ")
    }
}

fn main() {
    let p = Pair::new(3, 8);
    println!("{} -> {}", p, p.announce());
    let s = Pair::new("pear".to_string(), "apple".to_string());
    println!("{} -> {}", s, s.announce());
    let t = Pair::new(Celsius(21.5), Celsius(19.25));
    println!("{} -> {}", t, t.announce());
    let swapped = t.clone().swap();
    println!("swapped {} equal? {}", swapped, swapped == t);

    let mut temps = Bounded { items: Vec::new(), cap: 3 };
    for c in [12.0, 30.5, -4.0, 22.0] {
        if let Err(e) = temps.push(Celsius(c)) { println!("error: {}", e); }
    }
    println!("temps: {} max={}", temps.render(), temps.max().unwrap());
    let mut words: Bounded<&str> = Bounded { items: vec![], cap: 5 };
    for w in "kiwi fig banana".split(' ') { words.push(w).unwrap(); }
    println!("words: {} max={:?}", words.render(), words.max());
    let empty: Bounded<i32> = Bounded { items: vec![], cap: 1 };
    println!("empty max: {:?}", empty.max());
    println!("nan compare: {:?}", Celsius(f64::NAN).partial_cmp(&Celsius(1.0)));
}
