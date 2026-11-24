use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

struct D(String);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

trait Shape {
    fn area(&self) -> f64;
    fn name(&self) -> String;
}
struct Sq(f64, D);
impl Shape for Sq {
    fn area(&self) -> f64 { self.0 * self.0 }
    fn name(&self) -> String { format!("sq{}", self.0) }
}

struct Counter {
    counts: HashMap<String, usize>,
}
impl Counter {
    fn bump(&mut self, k: &str) -> usize {
        let c = self.counts.entry(k.to_string()).or_insert(0);
        *c += 1;
        *c
    }
}

fn longest_line<'a>(text: &'a str) -> &'a str {
    text.lines().max_by_key(|l| l.len()).unwrap_or("")
}

struct Holder<'a> {
    parts: Vec<&'a str>,
}
impl<'a> Holder<'a> {
    fn add(&mut self, s: &'a str) -> &mut Self {
        self.parts.push(s);
        self
    }
    fn longest(&self) -> &'a str {
        self.parts.iter().copied().max_by_key(|s| s.len()).unwrap_or("")
    }
}

fn main() {
    println!("-- for break");
    {
        let v = vec![D("f1".into()), D("f2".into()), D("f3".into())];
        for d in v {
            println!("visit {}", d.0);
            if d.0 == "f2" {
                break;
            }
        }
        println!("after loop");
    }
    println!("-- dyn vec");
    {
        let shapes: Vec<Box<dyn Shape>> = vec![Box::new(Sq(2.0, D("s2".into()))), Box::new(Sq(3.0, D("s3".into())))];
        let total: f64 = shapes.iter().map(|s| s.area()).sum();
        println!("total {} {}", total, shapes[1].name());
    }
    println!("-- rc refcell shared drop");
    {
        let shared = Rc::new(RefCell::new(vec![D("r1".into())]));
        let other = Rc::clone(&shared);
        other.borrow_mut().push(D("r2".into()));
        shared.borrow_mut().remove(0);
        println!("removed; count {}", Rc::strong_count(&shared));
    }
    println!("-- closure fnmut counter");
    let mut c = Counter { counts: HashMap::new() };
    let mut bump = |k: &str| c.bump(k);
    bump("a");
    bump("b");
    let n = bump("a");
    println!("n = {}", n);
    println!("a={} b={}", c.counts["a"], c.counts["b"]);

    println!("-- lifetimes");
    let text = String::from("short\na much longer line\nmid line");
    let l = longest_line(&text);
    println!("{}", l);
    let a = String::from("xx");
    let b = String::from("yyyy");
    let mut h = Holder { parts: vec![] };
    h.add(&a).add(&b).add(l);
    println!("{}", h.longest());

    println!("-- closures capturing then used after");
    let mut items = vec![3, 1, 2];
    let max = *items.iter().max().unwrap();
    let scaled: Vec<i32> = items.iter().map(|x| x * max).collect();
    items.push(max);
    println!("{:?} {:?}", items, scaled);
    let mut sum = 0;
    items.iter().for_each(|x| sum += x);
    println!("sum {}", sum);
    let names = vec!["b".to_string(), "a".to_string()];
    let mut sorted = names.clone();
    sorted.sort_by(|x, y| x.cmp(y));
    let contains = |s: &str| names.iter().any(|n| n == s);
    println!("{} {} {:?} {:?}", contains("a"), contains("z"), sorted, names);

    println!("-- option get_or_insert");
    let mut cache: Option<Vec<i32>> = None;
    cache.get_or_insert_with(Vec::new).push(1);
    cache.get_or_insert_with(Vec::new).push(2);
    println!("{:?}", cache);
    let mut slot: Option<D> = None;
    let r = slot.insert(D("ins".into()));
    r.0.push('!');
    println!("inserted {}", slot.as_ref().unwrap().0);
    println!("end");
}
