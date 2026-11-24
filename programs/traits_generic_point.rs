use std::fmt::{Debug, Display};
use std::ops::{Add, Mul, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Point<T> {
    x: T,
    y: T,
}

impl<T> Point<T> {
    fn new(x: T, y: T) -> Self {
        Point { x, y }
    }
}

impl<T: Copy + Add<Output = T> + Mul<Output = T>> Point<T> {
    fn dot(self, o: Point<T>) -> T {
        self.x * o.x + self.y * o.y
    }
}

impl Point<f64> {
    fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

impl<T: Add<Output = T>> Add for Point<T> {
    type Output = Point<T>;
    fn add(self, o: Point<T>) -> Point<T> {
        Point::new(self.x + o.x, self.y + o.y)
    }
}

impl<T> Sub for Point<T>
where
    T: Sub<Output = T>,
{
    type Output = Self;
    fn sub(self, o: Self) -> Self {
        Point { x: self.x - o.x, y: self.y - o.y }
    }
}

impl<T: Mul<Output = T> + Copy> Mul<T> for Point<T> {
    type Output = Point<T>;
    fn mul(self, k: T) -> Point<T> {
        Point::new(self.x * k, self.y * k)
    }
}

impl<T: Display> Display for Point<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

fn sum_points<T>(pts: &[Point<T>]) -> Point<T>
where
    T: Add<Output = T> + Copy + Default,
{
    pts.iter().fold(Point::default(), |acc, &p| acc + p)
}

fn centroid(pts: &[Point<f64>]) -> Point<f64> {
    sum_points(pts) * (1.0 / pts.len() as f64)
}

fn mixed<A: Into<f64> + Copy, B: Into<f64> + Copy>(p: Point<A>, q: Point<B>) -> Point<f64> {
    Point::new(p.x.into() + q.x.into(), p.y.into() + q.y.into())
}

fn largest<T: PartialOrd + Copy + Debug>(items: &[T]) -> T {
    let mut m = items[0];
    for &i in items {
        if i > m {
            m = i;
        }
    }
    m
}

fn main() {
    let a = Point::new(1, 2);
    let b = Point::new(3, -4);
    println!("{} {} {:?}", a + b, a - b, a * 3);
    println!("dot {}", a.dot(b));
    let fs = [Point::new(0.0, 0.0), Point::new(4.0, 0.0), Point::new(4.0, 3.0)];
    println!("sum {}", sum_points(&fs));
    println!("centroid {:.3}", centroid(&fs).length());
    println!("len {}", (fs[2] - fs[0]).length());
    println!("{}", sum_points(&[a, b, Point::new(10, 10)]));
    let m = mixed(Point::new(1u8, 2u8), Point::new(0.5f32, 0.25f32));
    println!("{:?}", m);
    println!("{} {} {:?}", largest(&[3, 9, 2]), largest(&[1.5, -2.0]), largest(&['x', 'b']));
    let pts: Vec<Point<i64>> = (1..=3).map(|i| Point::new(i, i * i)).collect();
    let far = pts.iter().copied().max_by_key(|p| p.dot(*p)).unwrap();
    println!("{} eq={}", far, far == Point::new(3, 9));
    let strs = Point::new(String::from("a"), String::from("b"));
    println!("{} {:?}", strs, Point::<u8>::default());
}
