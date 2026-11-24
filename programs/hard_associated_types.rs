use std::collections::BTreeMap;

trait Container {
    type Item;
    fn items(&self) -> Vec<Self::Item>;
    fn insert(&mut self, item: Self::Item);
    fn len(&self) -> usize { self.items().len() }
}

trait Transformer {
    type Input;
    type Output;
    fn apply(&self, x: Self::Input) -> Self::Output;
    fn apply_all<I>(&self, xs: I) -> Vec<Self::Output>
    where
        I: IntoIterator<Item = Self::Input>,
    {
        xs.into_iter().map(|x| self.apply(x)).collect()
    }
}

struct Bag { counts: BTreeMap<String, usize> }

impl Container for Bag {
    type Item = (String, usize);
    fn items(&self) -> Vec<(String, usize)> {
        self.counts.iter().map(|(k, v)| (k.clone(), *v)).collect()
    }
    fn insert(&mut self, (k, n): (String, usize)) {
        *self.counts.entry(k).or_insert(0) += n;
    }
}

struct Evens(Vec<u32>);

impl Container for Evens {
    type Item = u32;
    fn items(&self) -> Vec<u32> { self.0.clone() }
    fn insert(&mut self, x: u32) {
        if x % 2 == 0 { self.0.push(x); }
    }
}

struct Lengths;
impl Transformer for Lengths {
    type Input = &'static str;
    type Output = usize;
    fn apply(&self, s: &'static str) -> usize { s.chars().count() }
}

struct Scale(f64);
impl Transformer for Scale {
    type Input = i32;
    type Output = String;
    fn apply(&self, x: i32) -> String { format!("{:.1}", x as f64 * self.0) }
}

fn fill<C: Container>(c: &mut C, xs: Vec<C::Item>) -> usize {
    for x in xs { c.insert(x); }
    c.len()
}

fn sum_container<C>(c: &C) -> u64
where
    C: Container,
    C::Item: Into<u64>,
{
    c.items().into_iter().map(Into::into).sum()
}

fn main() {
    let mut bag = Bag { counts: BTreeMap::new() };
    let n = fill(&mut bag, vec![("pear".into(), 2), ("fig".into(), 1), ("pear".into(), 3)]);
    println!("bag has {} kinds: {:?}", n, bag.items());

    let mut ev = Evens(vec![]);
    let n = fill(&mut ev, (1..=10).collect());
    println!("evens kept {}: {:?} sum={}", n, ev.items(), sum_container(&ev));

    println!("lengths: {:?}", Lengths.apply_all(vec!["a", "héllo", "rust!"]));
    println!("scaled: {:?}", Scale(1.5).apply_all(1..=4));
    let s: <Scale as Transformer>::Output = Scale(-2.0).apply(7);
    println!("single: {}", s);
}
