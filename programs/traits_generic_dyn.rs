use std::collections::HashMap;
use std::fmt::Debug;

trait Observer<E> {
    fn notify(&mut self, event: &E) -> bool;
    fn summary(&self) -> String;
}

#[derive(Debug, Clone, PartialEq)]
enum Event {
    Click(i32, i32),
    Key(char),
    Quit,
}

struct Counter {
    counts: HashMap<String, u32>,
}

impl Observer<Event> for Counter {
    fn notify(&mut self, e: &Event) -> bool {
        let k = match e {
            Event::Click(..) => "click",
            Event::Key(_) => "key",
            Event::Quit => "quit",
        };
        *self.counts.entry(k.to_string()).or_default() += 1;
        true
    }
    fn summary(&self) -> String {
        let mut v: Vec<_> = self.counts.iter().collect();
        v.sort();
        format!("{:?}", v)
    }
}

struct Typist {
    text: String,
}

impl Observer<Event> for Typist {
    fn notify(&mut self, e: &Event) -> bool {
        if let Event::Key(c) = e {
            self.text.push(*c);
        }
        *e != Event::Quit
    }
    fn summary(&self) -> String {
        format!("typed {:?}", self.text)
    }
}

struct Logger<T: Debug> {
    seen: Vec<T>,
    limit: usize,
}

impl<T: Debug + Clone> Observer<T> for Logger<T> {
    fn notify(&mut self, e: &T) -> bool {
        if self.seen.len() < self.limit {
            self.seen.push(e.clone());
        }
        true
    }
    fn summary(&self) -> String {
        format!("first {}: {:?}", self.limit, self.seen)
    }
}

struct Bus<E> {
    observers: Vec<Box<dyn Observer<E>>>,
}

impl<E> Bus<E> {
    fn new() -> Self {
        Bus { observers: Vec::new() }
    }
    fn subscribe(&mut self, o: Box<dyn Observer<E>>) {
        self.observers.push(o);
    }
    fn publish(&mut self, e: E) -> usize {
        let mut keep = 0;
        for o in self.observers.iter_mut() {
            if o.notify(&e) {
                keep += 1;
            }
        }
        keep
    }
}

fn run<T: Debug + Clone>(obs: &mut dyn Observer<T>, items: &[T]) {
    for i in items {
        obs.notify(i);
    }
    println!("{}", obs.summary());
}

fn main() {
    let mut bus: Bus<Event> = Bus::new();
    bus.subscribe(Box::new(Counter { counts: HashMap::new() }));
    bus.subscribe(Box::new(Typist { text: String::new() }));
    bus.subscribe(Box::new(Logger { seen: vec![], limit: 2 }));
    let events = vec![Event::Key('h'), Event::Click(1, 2), Event::Key('i'), Event::Click(5, 5), Event::Quit];
    for e in events {
        let label = format!("{:?}", e);
        let n = bus.publish(e);
        println!("{} -> {} still listening", label, n);
    }
    for o in &bus.observers {
        println!("{}", o.summary());
    }
    let mut ints = Logger { seen: Vec::new(), limit: 3 };
    run(&mut ints, &[10, 20, 30, 40]);
    let mut strs: Logger<&str> = Logger { seen: vec![], limit: 1 };
    run(&mut strs, &["a", "b"]);
}
