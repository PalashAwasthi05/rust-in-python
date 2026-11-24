trait Animal {
    fn name(&self) -> String;
    fn sound(&self) -> &'static str;
    fn legs(&self) -> u32 { 4 }
    fn speak(&self) -> String {
        format!("{} says {}", self.name(), self.sound())
    }
    fn speak_n(&self, n: usize) -> String {
        let parts: Vec<&str> = std::iter::repeat(self.sound()).take(n).collect();
        format!("{}: {}", self.name(), parts.join("-"))
    }
    fn describe(&self) -> String {
        let kind = if self.legs() == 2 { "biped" } else if self.legs() == 0 { "legless" } else { "quadruped" };
        format!("{} ({}, {} legs) -> {}", self.name(), kind, self.legs(), self.speak())
    }
}

struct Dog { name: String }
struct Bird;
struct Snake { length_cm: u32 }

impl Animal for Dog {
    fn name(&self) -> String { self.name.clone() }
    fn sound(&self) -> &'static str { "woof" }
}

impl Animal for Bird {
    fn name(&self) -> String { "Bird".into() }
    fn sound(&self) -> &'static str { "tweet" }
    fn legs(&self) -> u32 { 2 }
    fn speak(&self) -> String { format!("{} sings {}!", self.name(), self.sound()) }
}

impl Animal for Snake {
    fn name(&self) -> String { format!("Snake[{}cm]", self.length_cm) }
    fn sound(&self) -> &'static str { "hiss" }
    fn legs(&self) -> u32 { 0 }
}

trait Scored {
    fn raw(&self) -> Vec<u32>;
    fn total(&self) -> u32 { self.raw().iter().sum() }
    fn best(&self) -> Option<u32> { self.raw().into_iter().max() }
    fn average(&self) -> f64 {
        let r = self.raw();
        if r.is_empty() { 0.0 } else { self.total() as f64 / r.len() as f64 }
    }
    fn summary(&self) -> String {
        format!("total={} best={:?} avg={:.2}", self.total(), self.best(), self.average())
    }
}

struct Round(Vec<u32>);
impl Scored for Round {
    fn raw(&self) -> Vec<u32> { self.0.clone() }
}

struct Capped { scores: Vec<u32>, cap: u32 }
impl Scored for Capped {
    fn raw(&self) -> Vec<u32> { self.scores.iter().map(|&s| s.min(self.cap)).collect() }
    fn best(&self) -> Option<u32> { Some(self.cap) }
}

fn main() {
    let zoo: Vec<Box<dyn Animal>> = vec![
        Box::new(Dog { name: "Rex".into() }),
        Box::new(Bird),
        Box::new(Snake { length_cm: 140 }),
    ];
    for a in &zoo {
        println!("{}", a.describe());
        println!("  {}", a.speak_n(3));
    }
    let total_legs: u32 = zoo.iter().map(|a| a.legs()).sum();
    println!("total legs: {}", total_legs);
    println!("{}", Round(vec![7, 3, 9, 1]).summary());
    println!("{}", Round(vec![]).summary());
    println!("{}", Capped { scores: vec![50, 120, 80], cap: 100 }.summary());
}
