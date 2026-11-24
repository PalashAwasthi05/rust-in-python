// Chapter 19.2: associated types, default generic parameters, operator overloading,
// fully qualified syntax, supertraits, and the newtype pattern.
use std::fmt;
use std::ops::{Add, Mul, Neg, Sub};

pub trait Container {
    type Item;
    fn get(&self, i: usize) -> Option<&Self::Item>;
    fn first(&self) -> Option<&Self::Item> {
        self.get(0)
    }
    fn size(&self) -> usize;
}

struct Shelf {
    books: Vec<String>,
}

impl Container for Shelf {
    type Item = String;
    fn get(&self, i: usize) -> Option<&String> {
        self.books.get(i)
    }
    fn size(&self) -> usize {
        self.books.len()
    }
}

fn describe<C: Container>(c: &C) -> String
where
    C::Item: fmt::Debug,
{
    format!("{} items, first = {:?}", c.size(), c.first())
}

struct Countdown(u32);

impl Iterator for Countdown {
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            None
        } else {
            self.0 -= 1;
            Some(self.0 + 1)
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

impl Add for Point {
    type Output = Point;
    fn add(self, other: Point) -> Point {
        Point { x: self.x + other.x, y: self.y + other.y }
    }
}

impl Sub for Point {
    type Output = Point;
    fn sub(self, other: Point) -> Point {
        Point { x: self.x - other.x, y: self.y - other.y }
    }
}

impl Mul<i32> for Point {
    type Output = Point;
    fn mul(self, k: i32) -> Point {
        Point { x: self.x * k, y: self.y * k }
    }
}

impl Neg for Point {
    type Output = Point;
    fn neg(self) -> Point {
        Point { x: -self.x, y: -self.y }
    }
}

#[derive(Debug, Clone, Copy)]
struct Millimeters(u32);
#[derive(Debug, Clone, Copy)]
struct Meters(u32);

impl Add<Meters> for Millimeters {
    type Output = Millimeters;
    fn add(self, other: Meters) -> Millimeters {
        Millimeters(self.0 + (other.0 * 1000))
    }
}

impl Add for Millimeters {
    type Output = Millimeters;
    fn add(self, other: Millimeters) -> Millimeters {
        Millimeters(self.0 + other.0)
    }
}

trait Pilot {
    fn fly(&self);
}

trait Wizard {
    fn fly(&self);
}

struct Human;

impl Pilot for Human {
    fn fly(&self) {
        println!("This is your captain speaking.");
    }
}

impl Wizard for Human {
    fn fly(&self) {
        println!("Up!");
    }
}

impl Human {
    fn fly(&self) {
        println!("*waving arms furiously*");
    }
}

trait Animal {
    fn baby_name() -> String;
}

struct Dog;

impl Dog {
    fn baby_name() -> String {
        String::from("Spot")
    }
}

impl Animal for Dog {
    fn baby_name() -> String {
        String::from("puppy")
    }
}

trait OutlinePrint: fmt::Display {
    fn outline_print(&self) {
        let output = self.to_string();
        let len = output.len();
        println!("{}", "*".repeat(len + 4));
        println!("*{}*", " ".repeat(len + 2));
        println!("* {output} *");
        println!("*{}*", " ".repeat(len + 2));
        println!("{}", "*".repeat(len + 4));
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl OutlinePrint for Point {}

struct Wrapper(Vec<String>);

impl fmt::Display for Wrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]", self.0.join(", "))
    }
}

impl std::ops::Deref for Wrapper {
    type Target = Vec<String>;
    fn deref(&self) -> &Vec<String> {
        &self.0
    }
}

trait Named {
    const KIND: &'static str;
    fn name(&self) -> String;
    fn label(&self) -> String {
        format!("{} {}", Self::KIND, self.name())
    }
}

impl Named for Dog {
    const KIND: &'static str = "dog";
    fn name(&self) -> String {
        String::from("Rex")
    }
}

impl Named for Human {
    const KIND: &'static str = "human";
    fn name(&self) -> String {
        String::from("Ada")
    }
    fn label(&self) -> String {
        format!("the {} called {}", Self::KIND, self.name())
    }
}

fn main() {
    let shelf = Shelf { books: vec![String::from("Dune"), String::from("Emma")] };
    println!("{}", describe(&shelf));
    println!("{:?} {:?}", shelf.get(1), shelf.get(5));
    let collected: Vec<u32> = Countdown(4).collect();
    println!("{:?}", collected);

    assert_eq!(Point { x: 1, y: 0 } + Point { x: 2, y: 3 }, Point { x: 3, y: 3 });
    let p = Point { x: 1, y: 2 } + Point { x: 10, y: 20 } * 2 - Point { x: 1, y: 1 };
    println!("{:?} neg {:?}", p, -p);
    let total = Millimeters(500) + Meters(2);
    println!("{:?}", total);
    println!("{:?}", total + Millimeters(1));

    let person = Human;
    Pilot::fly(&person);
    Wizard::fly(&person);
    person.fly();
    <Human as Wizard>::fly(&person);

    println!("A baby dog is called a {}", Dog::baby_name());
    println!("A baby dog is called a {}", <Dog as Animal>::baby_name());

    Point { x: 1, y: -3 }.outline_print();
    Point { x: 100, y: 2000 }.outline_print();

    let w = Wrapper(vec![String::from("hello"), String::from("world")]);
    println!("w = {w}");
    println!("len via Deref {}, first {:?}", w.len(), w.first());

    println!("{} / {}", Dog.label(), Human.label());
    println!("{} {}", <Dog as Named>::KIND, Human::KIND);
}
