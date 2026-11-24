use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Date {
    year: u16,
    month: u8,
    day: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Level {
    Debug,
    Info,
    Warn,
    Error,
}

fn date(year: u16, month: u8, day: u8) -> Date {
    Date { year, month, day }
}

fn main() {
    let mut events: BTreeMap<Date, Vec<&str>> = BTreeMap::new();
    events.entry(date(2024, 3, 1)).or_insert_with(Vec::new).push("launch");
    events.entry(date(2023, 12, 25)).or_insert_with(Vec::new).push("holiday");
    events.entry(date(2024, 1, 15)).or_insert_with(Vec::new).push("review");
    events.entry(date(2024, 3, 1)).or_insert_with(Vec::new).push("party");
    events.entry(date(2023, 2, 28)).or_insert_with(Vec::new).push("deadline");
    for (d, names) in &events {
        println!("{:04}-{:02}-{:02}: {}", d.year, d.month, d.day, names.join(", "));
    }
    println!("first: {:?}", events.keys().next());
    println!("last: {:?}", events.keys().last());
    println!("has 2024-01-15: {}", events.contains_key(&date(2024, 1, 15)));
    let after: Vec<String> = events
        .range(date(2024, 1, 1)..)
        .map(|(d, _)| format!("{}/{}", d.month, d.day))
        .collect();
    println!("in 2024: {:?}", after);

    let mut grid: BTreeMap<(i32, i32), char> = BTreeMap::new();
    grid.insert((2, 1), 'c');
    grid.insert((0, 5), 'a');
    grid.insert((2, -1), 'b');
    grid.insert((-1, 9), 'z');
    grid.insert((0, 0), 'o');
    for ((x, y), ch) in &grid {
        println!("({}, {}) = {}", x, y, ch);
    }
    let first_row: Vec<_> = grid.iter().filter(|((x, _), _)| *x == 0).map(|(_, c)| *c).collect();
    println!("x=0: {:?}", first_row);

    let mut by_level: BTreeMap<Level, usize> = BTreeMap::new();
    let logs = [Level::Warn, Level::Debug, Level::Error, Level::Warn, Level::Info, Level::Warn];
    for l in logs.iter() {
        *by_level.entry(*l).or_insert(0) += 1;
    }
    println!("{:?}", by_level);

    let mut set: BTreeSet<(String, u32)> = BTreeSet::new();
    set.insert(("bob".to_string(), 30));
    set.insert(("alice".to_string(), 40));
    set.insert(("bob".to_string(), 25));
    set.insert(("alice".to_string(), 40));
    set.insert(("carl".to_string(), 1));
    println!("{:?}", set);
    println!("len {}", set.len());
    println!("first {:?} last {:?}", set.first(), set.last());

    let a: BTreeSet<Level> = [Level::Info, Level::Error, Level::Debug].iter().cloned().collect();
    let b: BTreeSet<Level> = [Level::Warn, Level::Error].iter().cloned().collect();
    println!("union: {:?}", a.union(&b).collect::<Vec<_>>());
    println!("inter: {:?}", a.intersection(&b).collect::<Vec<_>>());
    println!("diff: {:?}", a.difference(&b).collect::<Vec<_>>());

    let mut nested: BTreeMap<Vec<u8>, Option<i32>> = BTreeMap::new();
    nested.insert(vec![1, 2, 3], Some(1));
    nested.insert(vec![1, 2], None);
    nested.insert(vec![0, 9, 9, 9], Some(-4));
    nested.insert(vec![], Some(0));
    nested.insert(vec![1, 3], Some(7));
    for (k, v) in &nested {
        println!("{:?} -> {:?}", k, v);
    }
    let mut opts: BTreeSet<Option<i32>> = BTreeSet::new();
    opts.insert(Some(3));
    opts.insert(None);
    opts.insert(Some(-2));
    println!("{:?}", opts);
}
