use std::fmt::Debug;

trait Plugin {
    fn name(&self) -> String;
    fn process(&self, input: &str) -> String;

    fn process_many(&self, inputs: &[&str]) -> Vec<String> {
        inputs.iter().map(|i| self.process(i)).collect()
    }

    fn with_label<L: Debug>(&self, label: L) -> String
    where
        Self: Sized,
    {
        format!("{:?}:{}", label, self.name())
    }

    fn boxed(self) -> Box<dyn Plugin>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }

    fn chain<P: Plugin>(self, next: P) -> Chain<Self, P>
    where
        Self: Sized,
    {
        Chain { first: self, second: next }
    }
}

struct Upper;
struct Reverse;
struct Repeat(usize);
struct Chain<A, B> { first: A, second: B }

impl Plugin for Upper {
    fn name(&self) -> String { "upper".into() }
    fn process(&self, s: &str) -> String { s.to_uppercase() }
}
impl Plugin for Reverse {
    fn name(&self) -> String { "reverse".into() }
    fn process(&self, s: &str) -> String { s.chars().rev().collect() }
}
impl Plugin for Repeat {
    fn name(&self) -> String { format!("repeat{}", self.0) }
    fn process(&self, s: &str) -> String { s.repeat(self.0) }
}
impl<A: Plugin, B: Plugin> Plugin for Chain<A, B> {
    fn name(&self) -> String { format!("{}+{}", self.first.name(), self.second.name()) }
    fn process(&self, s: &str) -> String { self.second.process(&self.first.process(s)) }
}

fn run_all(plugins: &[Box<dyn Plugin>], input: &str) {
    for p in plugins {
        println!("  {:<22} {:?}", p.name(), p.process(input));
    }
}

fn main() {
    println!("{}", Upper.with_label(1));
    println!("{}", Repeat(2).with_label("tag"));
    let combo = Upper.chain(Reverse).chain(Repeat(2));
    println!("{} -> {}", combo.name(), combo.process("abc"));

    let plugins: Vec<Box<dyn Plugin>> = vec![
        Upper.boxed(),
        Reverse.boxed(),
        Box::new(Repeat(3)),
        Reverse.chain(Upper).boxed(),
        combo.boxed(),
    ];
    println!("running {} plugins on \"rust\":", plugins.len());
    run_all(&plugins, "rust");
    let many = plugins[1].process_many(&["one", "two", "three"]);
    println!("process_many via dyn: {:?}", many);
    let names: Vec<String> = plugins.iter().map(|p| p.name()).collect();
    println!("names: {}", names.join(", "));
}
