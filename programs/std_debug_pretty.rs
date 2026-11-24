// Pretty debug output of nested structs, enums, tuples and maps.
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

#[derive(Debug, Clone, PartialEq)]
enum Shape {
    Circle { radius: f64 },
    Rect(u32, u32),
    Empty,
}

#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Debug)]
struct Unit;

#[derive(Debug)]
struct Wrapper(Vec<i32>);

#[derive(Debug)]
struct Scene {
    name: String,
    origin: Point,
    shapes: Vec<Shape>,
    tags: BTreeMap<String, Vec<u8>>,
    parent: Option<Box<Scene>>,
    marker: Unit,
}

fn main() {
    let mut tags = BTreeMap::new();
    tags.insert("b".to_string(), vec![1, 2]);
    tags.insert("a".to_string(), vec![]);
    let child = Scene {
        name: "child".into(),
        origin: Point { x: -1, y: 2 },
        shapes: vec![],
        tags: BTreeMap::new(),
        parent: None,
        marker: Unit,
    };
    let scene = Scene {
        name: "root \"main\"".to_string(),
        origin: Point { x: 0, y: 0 },
        shapes: vec![Shape::Circle { radius: 1.5 }, Shape::Rect(2, 3), Shape::Empty],
        tags,
        parent: Some(Box::new(child)),
        marker: Unit,
    };
    println!("{:?}", scene);
    println!("{:#?}", scene);
    println!("{:#?}", Wrapper(vec![7, 8]));
    println!("{:#?}", Wrapper(vec![]));
    println!("{:#?}", (Unit, Some(Point { x: 1, y: 1 }), Ok::<i32, String>(3)));
    let mut deque = VecDeque::new();
    deque.push_back(Shape::Empty);
    deque.push_front(Shape::Rect(1, 1));
    println!("{:#?}", deque);
    let set: BTreeSet<char> = "hello".chars().collect();
    println!("{:?}", set);
    println!("{:#?}", set);
    let mut hm: HashMap<u8, &str> = HashMap::new();
    hm.insert(1, "one");
    println!("{:?}", hm);
    println!("{:#?}", hm);
    let nested: Vec<Vec<Option<(i32, char)>>> = vec![vec![Some((1, 'a')), None], vec![]];
    println!("{:?}", nested);
    println!("{:#?}", nested);
    let arr = [[0u8; 2]; 2];
    println!("{:?} {:#?}", arr, ());
    let s: &[i32] = &[];
    println!("{:?} {:#?}", s, String::from("x"));
    println!("{:>10?}|{:<8?}|", Shape::Empty, Point { x: 1, y: 2 }.x);
}
