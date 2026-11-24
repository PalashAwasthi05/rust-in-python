// assert_eq! failure inside a helper after some output; exit status 101.
#[derive(Debug, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

fn midpoint(a: &Point, b: &Point) -> Point {
    Point { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 }
}

fn main() {
    let a = Point { x: 0, y: 0 };
    let b = Point { x: 4, y: 6 };
    let m = midpoint(&a, &b);
    assert!(m.x >= 0, "x should be non-negative");
    assert_eq!(m, Point { x: 2, y: 3 });
    assert_ne!(m.x, m.y);
    println!("midpoint ok: {:?}", m);
    let c = Point { x: 3, y: 3 };
    let m2 = midpoint(&a, &c);
    println!("second midpoint: {:?}", m2);
    assert_eq!(m2, Point { x: 2, y: 2 }, "rounding for {:?}", c);
    println!("never printed");
}
