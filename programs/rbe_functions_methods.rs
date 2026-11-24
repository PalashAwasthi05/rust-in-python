struct Point { x: f64, y: f64 }

impl Point {
    fn origin() -> Point { Point { x: 0.0, y: 0.0 } }
    fn new(x: f64, y: f64) -> Point { Point { x, y } }
    fn distance(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

struct Rectangle { p1: Point, p2: Point }

impl Rectangle {
    fn area(&self) -> f64 {
        let Point { x: x1, y: y1 } = self.p1;
        let Point { x: x2, y: y2 } = self.p2;
        ((x1 - x2) * (y1 - y2)).abs()
    }
    fn perimeter(&self) -> f64 {
        let Point { x: x1, y: y1 } = self.p1;
        let Point { x: x2, y: y2 } = self.p2;
        2.0 * ((x1 - x2).abs() + (y1 - y2).abs())
    }
    fn translate(&mut self, x: f64, y: f64) {
        self.p1.x += x;
        self.p2.x += x;
        self.p1.y += y;
        self.p2.y += y;
    }
}

struct Pair(Box<i32>, Box<i32>);

impl Pair {
    fn destroy(self) -> i32 {
        let Pair(first, second) = self;
        println!("destroying Pair({}, {})", first, second);
        *first + *second
    }
}

fn is_divisible_by(lhs: u32, rhs: u32) -> bool {
    if rhs == 0 { return false; }
    lhs % rhs == 0
}

fn fizzbuzz(n: u32) -> String {
    match (is_divisible_by(n, 3), is_divisible_by(n, 5)) {
        (true, true) => "fizzbuzz".into(),
        (true, false) => "fizz".into(),
        (false, true) => "buzz".into(),
        _ => n.to_string(),
    }
}

fn fizzbuzz_to(n: u32) -> Vec<String> {
    (1..=n).map(fizzbuzz).collect()
}

fn swap_and_sum(pair: &mut (i32, i32)) -> i32 {
    std::mem::swap(&mut pair.0, &mut pair.1);
    pair.0 + pair.1
}

fn apply_n(f: fn(u64) -> u64, n: usize, x: u64) -> u64 {
    (0..n).fold(x, |acc, _| f(acc))
}

fn main() {
    println!("{}", fizzbuzz_to(15).join(" "));
    let rectangle = Rectangle { p1: Point::origin(), p2: Point::new(3.0, 4.0) };
    println!("rectangle perimeter: {}", rectangle.perimeter());
    println!("rectangle area: {}", rectangle.area());
    println!("diagonal: {}", rectangle.p1.distance(&rectangle.p2));
    let mut square = Rectangle { p1: Point::origin(), p2: Point::new(1.0, 1.0) };
    square.translate(1.0, 1.0);
    println!("square moved to ({}, {})-({}, {}), area still {}", square.p1.x, square.p1.y, square.p2.x, square.p2.y, square.area());
    let pair = Pair(Box::new(1), Box::new(2));
    println!("sum from destroy: {}", pair.destroy());
    let mut t = (3, 9);
    println!("swap_and_sum {} -> {:?}", swap_and_sum(&mut t), t);
    fn double(x: u64) -> u64 { x * 2 }
    println!("apply double 10 times to 1: {}", apply_n(double, 10, 1));
    println!("Point::distance via path: {:.3}", Point::distance(&Point::new(1.0, 1.0), &Point::new(4.0, 5.0)));
    let method_ref: fn(&Rectangle) -> f64 = Rectangle::area;
    println!("method as fn pointer: {}", method_ref(&rectangle));
}
