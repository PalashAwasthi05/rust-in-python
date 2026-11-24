// Functions and methods returning `impl Iterator`, including ones that borrow from self.
struct Inventory {
    items: Vec<(String, u32, f64)>,
}

impl Inventory {
    fn in_stock(&self) -> impl Iterator<Item = &String> + '_ {
        self.items.iter().filter(|(_, qty, _)| *qty > 0).map(|(name, _, _)| name)
    }

    fn priced_above(&self, limit: f64) -> impl Iterator<Item = (&str, f64)> + '_ {
        self.items
            .iter()
            .filter(move |item| item.2 > limit)
            .map(|item| (item.0.as_str(), item.2))
    }

    fn values(&self) -> impl Iterator<Item = f64> + '_ {
        self.items.iter().map(|(_, q, p)| *q as f64 * p)
    }
}

fn evens(limit: u32) -> impl Iterator<Item = u32> {
    (0..limit).filter(|n| n % 2 == 0)
}

fn squares_of<'a>(xs: &'a [i64]) -> impl Iterator<Item = i64> + 'a {
    xs.iter().map(|x| x * x)
}

fn words_longer_than(text: &str, n: usize) -> impl Iterator<Item = &str> {
    text.split_whitespace().filter(move |w| w.len() > n)
}

fn counter_from(start: i32) -> impl Iterator<Item = i32> {
    let mut current = start;
    std::iter::from_fn(move || {
        current += 1;
        Some(current)
    })
}

fn compose<I>(it: I) -> impl Iterator<Item = String>
where
    I: Iterator<Item = u32>,
{
    it.map(|x| x * 3).filter(|x| x % 2 == 1).map(|x| format!("<{}>", x))
}

fn main() {
    let inv = Inventory {
        items: vec![
            ("apple".to_string(), 3, 0.5),
            ("pear".to_string(), 0, 0.75),
            ("melon".to_string(), 2, 3.25),
            ("kiwi".to_string(), 10, 0.2),
        ],
    };
    let names: Vec<&String> = inv.in_stock().collect();
    println!("in stock: {:?}", names);
    for (name, price) in inv.priced_above(0.4) {
        println!("{} costs {:.2}", name, price);
    }
    let total: f64 = inv.values().sum();
    println!("total value {:.2}", total);

    let e: Vec<u32> = evens(10).collect();
    println!("evens {:?}", e);
    println!("sum of squares {}", squares_of(&[1, 2, 3, 4]).sum::<i64>());
    let long: Vec<&str> = words_longer_than("the quick brown fox jumps over lazy dogs", 4).collect();
    println!("long words {:?}", long);
    let c: Vec<i32> = counter_from(10).take(4).collect();
    println!("counter {:?}", c);
    let composed: Vec<String> = compose(1..8).collect();
    println!("composed {}", composed.join(""));
    let nested: u32 = compose(evens(20).map(|x| x + 1)).count() as u32;
    println!("nested count {}", nested);
}
