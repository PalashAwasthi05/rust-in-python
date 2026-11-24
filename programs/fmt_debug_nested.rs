use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

#[derive(Debug)]
struct Unit;

#[derive(Debug)]
struct Pair(i32, &'static str);

#[derive(Debug)]
struct Empty {}

#[derive(Debug)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(Debug)]
enum Shape {
    Circle { center: Point, radius: f64 },
    Square(f64),
    Nothing,
}

#[derive(Debug)]
struct Wrapper<T>(T);

fn main() {
    println!("{:?}", Unit);
    println!("{:?}", Pair(1, "one"));
    println!("{:?}", Empty {});
    println!("{:?}", Point { x: 1.0, y: -2.5 });
    println!("{:?}", Shape::Circle { center: Point { x: 0.0, y: 0.0 }, radius: 1.5 });
    println!("{:?}", Shape::Square(2.0));
    println!("{:?}", Shape::Nothing);
    println!("{:?}", Wrapper(Wrapper(vec![Some(1), None])));
    let nested = vec![vec![1, 2], vec![], vec![3]];
    println!("{:?}", nested);
    let tuple = (1, "two", 3.0, '4', true, ());
    println!("{:?}", tuple);
    println!("{:?}", (5,));
    println!("{:?}", ((1, 2), [3, 4], (vec![5],)));
    let opt: Option<Option<i32>> = Some(None);
    println!("{:?}", opt);
    let res: Result<i32, String> = Err("bad".to_string());
    println!("{:?}", res);
    let ok: Result<(), String> = Ok(());
    println!("{:?}", ok);
    let mut hm = HashMap::new();
    hm.insert("only", vec![1, 2]);
    println!("{:?}", hm);
    let mut bt = BTreeMap::new();
    bt.insert(3, "c");
    bt.insert(1, "a");
    bt.insert(2, "b");
    println!("{:?}", bt);
    let bs: BTreeSet<char> = "hello".chars().collect();
    println!("{:?}", bs);
    let mut hs = HashSet::new();
    hs.insert(42);
    println!("{:?}", hs);
    let dq: VecDeque<i32> = (1..4).collect();
    println!("{:?}", dq);
    let empty: Vec<i32> = Vec::new();
    println!("{:?} {:?}", empty, BTreeMap::<i32, i32>::new());
    let boxed: Box<Vec<i32>> = Box::new(vec![1, 2]);
    println!("{:?}", boxed);
    let arr = [[0u8; 3]; 2];
    println!("{:?}", arr);
    let slice = &[1.5, 2.0][..];
    println!("{:?}", slice);
    let s = String::from("hi");
    let r = &&s;
    println!("{:?} {}", r, r);
    println!("{:?}", Box::new(Point { x: 3.0, y: 4.0 }));
    println!("{:?}", std::rc::Rc::new(5));
    println!("{:?}", std::cell::RefCell::new(vec![1]));
    println!("{:?}", std::cell::Cell::new(2));
    println!("{:?}", 'a'..'e');
    println!("{:?}", 1..=5);
    println!("{:?}", "a b c".split(' ').collect::<Vec<_>>());
}
