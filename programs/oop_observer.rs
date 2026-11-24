// Observer pattern with Vec<Box<dyn Fn(&Event)>>, FnMut subscribers and Rc<RefCell> listeners.
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

#[derive(Debug, Clone, PartialEq)]
enum Event {
    Click { x: i32, y: i32 },
    Key(char),
    Resize(u32, u32),
    Quit,
}

impl Event {
    fn kind(&self) -> &'static str {
        match self {
            Event::Click { .. } => "click",
            Event::Key(_) => "key",
            Event::Resize(..) => "resize",
            Event::Quit => "quit",
        }
    }
}

struct EventBus {
    listeners: Vec<Box<dyn Fn(&Event)>>,
    by_kind: HashMap<&'static str, Vec<Box<dyn Fn(&Event) -> String>>>,
}

impl EventBus {
    fn new() -> Self {
        EventBus { listeners: Vec::new(), by_kind: HashMap::new() }
    }

    fn subscribe<F: Fn(&Event) + 'static>(&mut self, f: F) {
        self.listeners.push(Box::new(f));
    }

    fn on(&mut self, kind: &'static str, f: impl Fn(&Event) -> String + 'static) {
        self.by_kind.entry(kind).or_insert_with(Vec::new).push(Box::new(f));
    }

    fn emit(&self, event: &Event) {
        for l in &self.listeners {
            l(event);
        }
        if let Some(handlers) = self.by_kind.get(event.kind()) {
            for (i, h) in handlers.iter().enumerate() {
                println!("  handler {} for {}: {}", i, event.kind(), h(event));
            }
        }
    }
}

// FnMut subscribers that mutate captured state.
struct Counter<'a> {
    subscribers: Vec<Box<dyn FnMut(i32) + 'a>>,
}

impl<'a> Counter<'a> {
    fn notify(&mut self, v: i32) {
        for s in self.subscribers.iter_mut() {
            s(v);
        }
    }
}

// Classic observer with trait objects and weak back-references.
trait Observer {
    fn update(&self, subject: &str, value: i32);
    fn id(&self) -> u32;
}

struct Logger {
    id: u32,
    log: RefCell<Vec<String>>,
}

impl Observer for Logger {
    fn update(&self, subject: &str, value: i32) {
        self.log.borrow_mut().push(format!("{}={}", subject, value));
    }
    fn id(&self) -> u32 {
        self.id
    }
}

struct Subject {
    name: String,
    value: i32,
    observers: Vec<Weak<dyn Observer>>,
}

impl Subject {
    fn attach(&mut self, o: &Rc<dyn Observer>) {
        self.observers.push(Rc::downgrade(o));
    }
    fn set(&mut self, v: i32) {
        self.value = v;
        self.observers.retain(|w| w.upgrade().is_some());
        for w in &self.observers {
            if let Some(o) = w.upgrade() {
                o.update(&self.name, v);
            }
        }
        println!("{} set to {}, {} live observers", self.name, v, self.observers.len());
    }
}

fn main() {
    let seen = Rc::new(RefCell::new(Vec::<String>::new()));
    let mut bus = EventBus::new();
    bus.subscribe(|e| println!("log: {:?}", e));
    let sink = Rc::clone(&seen);
    bus.subscribe(move |e| sink.borrow_mut().push(e.kind().to_string()));
    bus.on("click", |e| match e {
        Event::Click { x, y } => format!("clicked at ({}, {})", x, y),
        _ => unreachable!(),
    });
    bus.on("click", |e| if let Event::Click { x, .. } = e { format!("left half: {}", *x < 50) } else { String::new() });
    bus.on("key", |e| format!("{:?} pressed", e));

    let events = vec![Event::Click { x: 10, y: 20 }, Event::Key('q'), Event::Resize(800, 600), Event::Click { x: 70, y: 1 }, Event::Quit];
    for e in &events {
        bus.emit(e);
    }
    println!("seen kinds: {:?}", seen.borrow());
    println!("rc count: {}", Rc::strong_count(&seen));

    let mut total = 0;
    let mut history = Vec::new();
    {
        let mut counter = Counter { subscribers: Vec::new() };
        counter.subscribers.push(Box::new(|v| total += v));
        counter.subscribers.push(Box::new(|v| history.push(v * 10)));
        for v in [1, 2, 3, 4] {
            counter.notify(v);
        }
    }
    println!("total={} history={:?}", total, history);

    let a: Rc<dyn Observer> = Rc::new(Logger { id: 1, log: RefCell::new(vec![]) });
    let mut subject = Subject { name: "temp".into(), value: 0, observers: vec![] };
    subject.attach(&a);
    {
        let b: Rc<dyn Observer> = Rc::new(Logger { id: 2, log: RefCell::new(vec![]) });
        subject.attach(&b);
        subject.set(21);
        println!("observer {} alive", b.id());
    }
    subject.set(25);
    println!("observer {} saw value {}", a.id(), subject.value);
}
