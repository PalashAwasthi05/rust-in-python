#[derive(Debug, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

fn mirror(p: &Point) -> Point {
    Point { x: -p.x, y: p.y }
}

fn main() {
    let p = Point { x: 3, y: 4 };
    assert_eq!(mirror(&mirror(&p)), p);
    assert!(p.x < p.y, "x should be smaller");
    assert_ne!(p.x, p.y);
    println!("basic checks passed");
    let words = vec!["alpha", "beta"];
    assert_eq!(words.len(), 2, "two words expected");
    println!("about to fail");
    assert_eq!(mirror(&p), Point { x: 3, y: 4 }, "mirror of {:?} should be unchanged", p);
    println!("unreachable");
}
