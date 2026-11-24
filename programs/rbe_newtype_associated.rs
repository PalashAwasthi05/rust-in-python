use std::fmt;
use std::ops::Add;

struct Years(i64);
struct Days(i64);

impl Years {
    pub fn to_days(&self) -> Days { Days(self.0 * 365) }
}

impl Days {
    pub fn to_years(&self) -> Years { Years(self.0 / 365) }
}

fn is_adult(age: &Years) -> bool { age.0 >= 18 }

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Meters(f64);

impl Add for Meters {
    type Output = Meters;
    fn add(self, o: Meters) -> Meters { Meters(self.0 + o.0) }
}

struct Wrapper(Vec<String>);

impl fmt::Display for Wrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]", self.0.join(", "))
    }
}

// associated types make the container's element types part of the trait
struct Container(i32, i32);

trait Contains {
    type A;
    type B;
    fn contains(&self, _: &Self::A, _: &Self::B) -> bool;
    fn first(&self) -> i32;
    fn last(&self) -> i32;
}

impl Contains for Container {
    type A = i32;
    type B = i32;
    fn contains(&self, number_1: &i32, number_2: &i32) -> bool {
        (&self.0 == number_1) && (&self.1 == number_2)
    }
    fn first(&self) -> i32 { self.0 }
    fn last(&self) -> i32 { self.1 }
}

fn difference<C: Contains>(container: &C) -> i32 {
    container.last() - container.first()
}

trait Unit {
    const SYMBOL: &'static str;
    type Raw;
    fn raw(&self) -> Self::Raw;
    fn label(&self) -> String where Self::Raw: fmt::Display {
        format!("{}{}", self.raw(), Self::SYMBOL)
    }
}

impl Unit for Meters {
    const SYMBOL: &'static str = "m";
    type Raw = f64;
    fn raw(&self) -> f64 { self.0 }
}

impl Unit for Years {
    const SYMBOL: &'static str = "y";
    type Raw = i64;
    fn raw(&self) -> i64 { self.0 }
}

fn main() {
    let age = Years(25);
    let age_days = age.to_days();
    println!("old enough {}", is_adult(&age));
    println!("old enough {}", is_adult(&age_days.to_years()));
    println!("{} years is {} days", age.0, age_days.0);
    let Years(raw) = Days(800).to_years();
    println!("800 days is {} whole years", raw);

    let total = Meters(1.5) + Meters(2.25);
    println!("{:?}, larger than 3m: {}", total, total > Meters(3.0));
    println!("labels: {} {}", total.label(), Years(4).label());
    let w = Wrapper(vec![String::from("hello"), String::from("world")]);
    println!("w = {}", w);

    let number_1 = 3;
    let number_2 = 10;
    let container = Container(number_1, number_2);
    println!("does container contain {} and {}: {}", &number_1, &number_2, container.contains(&number_1, &number_2));
    println!("first number: {}", container.first());
    println!("last number: {}", container.last());
    println!("the difference is: {}", difference(&container));
}
