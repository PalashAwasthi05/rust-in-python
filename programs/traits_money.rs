use std::fmt;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, Mul, Sub};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
struct Cents(i64);

impl Add for Cents {
    type Output = Cents;
    fn add(self, o: Cents) -> Cents {
        Cents(self.0 + o.0)
    }
}

impl Sub for Cents {
    type Output = Cents;
    fn sub(self, o: Cents) -> Cents {
        Cents(self.0 - o.0)
    }
}

impl Mul<u32> for Cents {
    type Output = Cents;
    fn mul(self, q: u32) -> Cents {
        Cents(self.0 * q as i64)
    }
}

impl Div<i64> for Cents {
    type Output = (Cents, Cents);
    fn div(self, n: i64) -> (Cents, Cents) {
        (Cents(self.0 / n), Cents(self.0 % n))
    }
}

impl AddAssign for Cents {
    fn add_assign(&mut self, o: Cents) {
        self.0 += o.0;
    }
}

impl Sum for Cents {
    fn sum<I: Iterator<Item = Cents>>(iter: I) -> Cents {
        iter.fold(Cents(0), |a, b| a + b)
    }
}

impl<'a> Sum<&'a Cents> for Cents {
    fn sum<I: Iterator<Item = &'a Cents>>(iter: I) -> Cents {
        iter.copied().sum()
    }
}

impl fmt::Display for Cents {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let sign = if self.0 < 0 { "-" } else { "" };
        write!(f, "{}${}.{:02}", sign, self.0.abs() / 100, self.0.abs() % 100)
    }
}

#[derive(Debug, PartialEq)]
struct ParseMoneyError(String);

impl FromStr for Cents {
    type Err = ParseMoneyError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_prefix('$').ok_or_else(|| ParseMoneyError(format!("missing $ in {:?}", s)))?;
        let (d, c) = s.split_once('.').unwrap_or((s, "0"));
        let dollars: i64 = d.parse().map_err(|_| ParseMoneyError(format!("bad dollars {:?}", d)))?;
        let cents: i64 = c.parse().map_err(|_| ParseMoneyError(format!("bad cents {:?}", c)))?;
        Ok(Cents(dollars * 100 + cents))
    }
}

struct LineItem {
    name: &'static str,
    price: Cents,
    qty: u32,
}

fn main() {
    let items = vec![
        LineItem { name: "coffee", price: "$3.50".parse().unwrap(), qty: 2 },
        LineItem { name: "bagel", price: "$2.25".parse().unwrap(), qty: 3 },
        LineItem { name: "juice", price: Cents::from_str("$4").unwrap(), qty: 1 },
    ];
    let mut total = Cents::default();
    for it in &items {
        let line = it.price * it.qty;
        println!("{:<8}{:>3} x {:>6} = {:>7}", it.name, it.qty, it.price.to_string(), line.to_string());
        total += line;
    }
    println!("total {}", total);
    let prices: Vec<Cents> = items.iter().map(|i| i.price).collect();
    let s1: Cents = prices.iter().sum();
    let s2: Cents = prices.into_iter().sum();
    println!("{} {}", s1, s2);
    let (each, rem) = total / 3;
    println!("split 3 ways: {} each, {} left", each, rem);
    println!("{}", Cents(5) - Cents(250));
    for bad in ["3.50", "$x.10", "$1.zz"] {
        println!("{:?}", bad.parse::<Cents>());
    }
    let cheapest = items.iter().min_by_key(|i| i.price).unwrap();
    println!("cheapest {}", cheapest.name);
    println!("{}", Cents(199) > Cents(198));
}
