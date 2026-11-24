// Inherent and trait impl blocks in a different module than the type.
mod model {
    #[derive(Debug, Clone, PartialEq)]
    pub struct Point {
        pub x: i32,
        pub y: i32,
    }

    pub enum Shape {
        Dot(Point),
        Line(Point, Point),
    }
}

mod geometry {
    use crate::model::{Point, Shape};

    impl Point {
        pub fn origin() -> Self {
            Point { x: 0, y: 0 }
        }

        pub fn manhattan(&self, other: &Point) -> i32 {
            (self.x - other.x).abs() + (self.y - other.y).abs()
        }
    }

    impl Shape {
        pub fn length(&self) -> i32 {
            match self {
                Shape::Dot(_) => 0,
                Shape::Line(a, b) => a.manhattan(b),
            }
        }
    }
}

mod display {
    use std::fmt;
    use super::model::Point;

    impl fmt::Display for Point {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "({}, {})", self.x, self.y)
        }
    }

    impl fmt::Display for crate::model::Shape {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                super::model::Shape::Dot(p) => write!(f, "dot at {}", p),
                super::model::Shape::Line(a, b) => write!(f, "line {} -> {}", a, b),
            }
        }
    }
}

use model::{Point, Shape};

fn main() {
    let o = Point::origin();
    let p = Point { x: 3, y: -4 };
    println!("{} {} {}", o, p, o.manhattan(&p));
    let shapes = vec![Shape::Dot(p.clone()), Shape::Line(o.clone(), p.clone())];
    for s in &shapes {
        println!("{} has length {}", s, s.length());
    }
    println!("{:?} {}", p, p == Point { x: 3, y: -4 });
}
