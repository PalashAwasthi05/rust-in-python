use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Person {
    last: String,
    first: String,
    age: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Point(f64, f64);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct Meters(i64);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Unit;

fn v(major: u32, minor: u32, patch: u32) -> Version {
    Version { major, minor, patch }
}

fn main() {
    let a = v(1, 2, 3);
    let b = v(1, 10, 0);
    let c = v(1, 2, 3);
    println!("{:?} == {:?}: {}", a, c, a == c);
    println!("{:?} != {:?}: {}", a, b, a != b);
    println!("a < b: {}", a < b);
    println!("a <= c: {}", a <= c);
    println!("b > a: {}", b > a);
    println!("a.cmp(&b): {:?}", a.cmp(&b));
    println!("b.cmp(&a): {:?}", b.cmp(&a));
    println!("a.cmp(&c): {:?}", a.cmp(&c));
    println!("partial: {:?}", a.partial_cmp(&b));

    let mut versions = vec![v(2, 0, 0), v(1, 10, 1), v(1, 2, 3), v(0, 9, 9), v(1, 10, 0), v(1, 2, 3)];
    versions.sort();
    for version in &versions {
        println!("{}.{}.{}", version.major, version.minor, version.patch);
    }
    println!("max: {:?}", versions.iter().max());
    println!("min: {:?}", versions.iter().min());
    versions.dedup();
    println!("after dedup: {}", versions.len());

    let mut people = vec![
        Person { last: "Smith".to_string(), first: "John".to_string(), age: 40 },
        Person { last: "Doe".to_string(), first: "Jane".to_string(), age: 31 },
        Person { last: "Smith".to_string(), first: "Anna".to_string(), age: 22 },
        Person { last: "Doe".to_string(), first: "Jane".to_string(), age: 29 },
        Person { last: "Adams".to_string(), first: "Zed".to_string(), age: 50 },
    ];
    people.sort();
    for p in &people {
        println!("{} {} {}", p.first, p.last, p.age);
    }
    people.sort_by(|x, y| y.cmp(x));
    let names: Vec<String> = people.iter().map(|p| format!("{}/{}", p.last, p.age)).collect();
    println!("desc: {:?}", names);

    let p = Point(1.0, 2.0);
    let q = Point(1.0, f64::NAN);
    let r = Point(0.5, 100.0);
    println!("p < r: {}", p < r);
    println!("r < p: {}", r < p);
    println!("p vs q: {:?}", p.partial_cmp(&q));
    println!("q == q: {}", q == q);
    println!("p == p: {}", p == p);
    println!("p <= q: {}, p >= q: {}", p <= q, p >= q);

    let d = [Meters(5), Meters(-3), Meters(12)];
    let longest = d.iter().max().unwrap();
    println!("longest: {:?}", longest);
    println!("Unit == Unit: {}, cmp: {:?}", Unit == Unit, Unit.cmp(&Unit));
    println!("ordering lt: {}", Ordering::Less < Ordering::Equal);
}
