use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Severity { Low, Medium, High, Critical }

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Ticket {
    severity: Severity,
    age_days: u32,
    title: String,
}

#[derive(Debug, PartialEq, Eq)]
struct Job { priority: u8, seq: u32, name: &'static str }

impl Ord for Job {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority).then_with(|| other.seq.cmp(&self.seq))
    }
}
impl PartialOrd for Job {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

#[derive(Debug, PartialEq, PartialOrd)]
enum Shape { Dot, Circle(f64), Rect { w: f64, h: f64 } }

fn main() {
    let mut sev = vec![Severity::High, Severity::Low, Severity::Critical, Severity::Medium, Severity::Low];
    sev.sort();
    println!("sorted severities: {:?}", sev);
    println!("High > Medium: {}, max: {:?}", Severity::High > Severity::Medium, sev.iter().max());

    let mut tickets = vec![
        Ticket { severity: Severity::High, age_days: 3, title: "login fails".into() },
        Ticket { severity: Severity::Low, age_days: 30, title: "typo".into() },
        Ticket { severity: Severity::High, age_days: 3, title: "data loss".into() },
        Ticket { severity: Severity::Critical, age_days: 1, title: "outage".into() },
        Ticket { severity: Severity::High, age_days: 10, title: "slow page".into() },
    ];
    tickets.sort();
    for t in &tickets { println!("  {:?} {:>2}d {}", t.severity, t.age_days, t.title); }
    tickets.sort_by(|a, b| b.cmp(a));
    println!("top ticket: {}", tickets[0].title);

    let mut heap: BinaryHeap<Ticket> = tickets.into_iter().collect();
    print!("heap order:");
    while let Some(t) = heap.pop() { print!(" [{}]", t.title); }
    println!();

    let mut jobs = BinaryHeap::new();
    for (seq, (p, name)) in [(1, "backup"), (5, "deploy"), (5, "hotfix"), (3, "report"), (1, "cleanup")].iter().enumerate() {
        jobs.push(Job { priority: *p, seq: seq as u32, name });
    }
    println!("peek: {:?}", jobs.peek().map(|j| j.name));
    let order: Vec<&str> = std::iter::from_fn(|| jobs.pop()).map(|j| j.name).collect();
    println!("job order: {:?}", order);

    let mut mins = BinaryHeap::new();
    for x in [5, 1, 8, 3, 9, 2] { mins.push(Reverse(x)); }
    let smallest3: Vec<i32> = (0..3).filter_map(|_| mins.pop()).map(|Reverse(x)| x).collect();
    println!("3 smallest: {:?}", smallest3);
    println!("into_sorted_vec: {:?}", BinaryHeap::from(vec![4, 1, 7, 3]).into_sorted_vec());

    println!("Dot < Circle(0): {}", Shape::Dot < Shape::Circle(0.0));
    println!("Circle(2) vs Circle(1): {:?}", Shape::Circle(2.0).partial_cmp(&Shape::Circle(1.0)));
    println!("Rect cmp: {:?}", Shape::Rect { w: 1.0, h: 5.0 }.partial_cmp(&Shape::Rect { w: 1.0, h: 2.0 }));
    println!("NaN cmp: {:?}", Shape::Circle(f64::NAN).partial_cmp(&Shape::Circle(1.0)));
    println!("tuple ord: {:?}", (2, "b").cmp(&(2, "a")));
}
