// Strategy pattern via trait objects and via generics; also closures as strategies.
trait Discount {
    fn apply(&self, price: f64) -> f64;
    fn name(&self) -> String;
}

struct NoDiscount;
struct Percent(f64);
struct Flat {
    amount: f64,
    min_price: f64,
}

impl Discount for NoDiscount {
    fn apply(&self, price: f64) -> f64 {
        price
    }
    fn name(&self) -> String {
        "none".into()
    }
}

impl Discount for Percent {
    fn apply(&self, price: f64) -> f64 {
        price * (1.0 - self.0 / 100.0)
    }
    fn name(&self) -> String {
        format!("{}% off", self.0)
    }
}

impl Discount for Flat {
    fn apply(&self, price: f64) -> f64 {
        if price >= self.min_price {
            price - self.amount
        } else {
            price
        }
    }
    fn name(&self) -> String {
        format!("${} off over ${}", self.amount, self.min_price)
    }
}

struct Checkout {
    strategy: Box<dyn Discount>,
}

impl Checkout {
    fn set_strategy(&mut self, s: Box<dyn Discount>) {
        self.strategy = s;
    }
    fn total(&self, items: &[f64]) -> f64 {
        let sum: f64 = items.iter().sum();
        self.strategy.apply(sum)
    }
}

// Generic (static dispatch) strategy.
trait SortStrategy {
    fn sort(&self, data: &mut Vec<i32>);
}

struct Ascending;
struct Descending;
struct ByAbs;

impl SortStrategy for Ascending {
    fn sort(&self, data: &mut Vec<i32>) {
        data.sort();
    }
}
impl SortStrategy for Descending {
    fn sort(&self, data: &mut Vec<i32>) {
        data.sort_by(|a, b| b.cmp(a));
    }
}
impl SortStrategy for ByAbs {
    fn sort(&self, data: &mut Vec<i32>) {
        data.sort_by_key(|x| x.abs());
    }
}

struct Sorter<S: SortStrategy> {
    strategy: S,
}

impl<S: SortStrategy> Sorter<S> {
    fn run(&self, mut data: Vec<i32>) -> Vec<i32> {
        self.strategy.sort(&mut data);
        data
    }
}

fn sort_with<S>(s: &S, data: &[i32]) -> Vec<i32>
where
    S: SortStrategy + ?Sized,
{
    let mut v = data.to_vec();
    s.sort(&mut v);
    v
}

// Closure-based strategy.
struct Validator<F: Fn(&str) -> bool> {
    rule: F,
    label: &'static str,
}

impl<F: Fn(&str) -> bool> Validator<F> {
    fn check(&self, input: &str) -> String {
        format!("{} {:?}: {}", self.label, input, if (self.rule)(input) { "ok" } else { "rejected" })
    }
}

fn main() {
    let items = [19.99, 5.01, 75.0];
    let mut c = Checkout { strategy: Box::new(NoDiscount) };
    println!("{}: {:.2}", c.strategy.name(), c.total(&items));
    c.set_strategy(Box::new(Percent(10.0)));
    println!("{}: {:.2}", c.strategy.name(), c.total(&items));
    c.set_strategy(Box::new(Flat { amount: 20.0, min_price: 50.0 }));
    println!("{}: {:.2}", c.strategy.name(), c.total(&items));
    println!("{}: {:.2}", c.strategy.name(), c.total(&items[..2]));

    let strategies: Vec<Box<dyn Discount>> =
        vec![Box::new(NoDiscount), Box::new(Percent(25.0)), Box::new(Flat { amount: 1.5, min_price: 0.0 })];
    let best = strategies
        .iter()
        .min_by(|a, b| a.apply(100.0).partial_cmp(&b.apply(100.0)).unwrap())
        .unwrap();
    println!("best for 100: {} -> {}", best.name(), best.apply(100.0));

    let data = vec![3, -7, 1, -2, 9, 0];
    println!("{:?}", Sorter { strategy: Ascending }.run(data.clone()));
    println!("{:?}", Sorter { strategy: Descending }.run(data.clone()));
    println!("{:?}", Sorter { strategy: ByAbs }.run(data.clone()));
    println!("{:?}", sort_with(&Descending, &data));
    let dynamic: &dyn SortStrategy = &ByAbs;
    println!("{:?}", sort_with(dynamic, &data));

    let non_empty = Validator { rule: |s: &str| !s.is_empty(), label: "non-empty" };
    let min_len = 4;
    let long_enough = Validator { rule: move |s: &str| s.len() >= min_len, label: "min-len" };
    for input in ["", "abc", "abcdef"] {
        println!("{} | {}", non_empty.check(input), long_enough.check(input));
    }

    let ops: Vec<(&str, fn(i64, i64) -> i64)> =
        vec![("add", |a, b| a + b), ("mul", |a, b| a * b), ("max", std::cmp::max)];
    for (name, op) in &ops {
        println!("{}(6, 7) = {}", name, op(6, 7));
    }
}
