use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone)]
struct Item {
    name: String,
    tags: Vec<&'static str>,
    price: Option<f64>,
}

#[derive(Debug)]
enum Event {
    Start,
    Move(i32, i32),
    Say { text: String, loud: bool },
}

#[derive(Debug)]
struct Unit;

#[derive(Debug)]
struct Tuple(u8, Unit, Vec<u8>);

fn main() {
    let item = Item { name: "apple".to_string(), tags: vec!["fruit", "red"], price: Some(1.25) };
    println!("{:#?}", item);
    let mut catalog: BTreeMap<&str, Vec<Item>> = BTreeMap::new();
    catalog.insert("fruit", vec![item.clone(), Item { name: "pear".into(), tags: vec![], price: None }]);
    catalog.insert("empty", vec![]);
    println!("{:#?}", catalog);
    let events = vec![Event::Start, Event::Move(1, -1), Event::Say { text: "hi\n".into(), loud: true }];
    println!("{:#?}", events);
    println!("{:#?}", Tuple(7, Unit, vec![1, 2]));
    println!("{:#?}", (1, "a"));
    println!("{:#?}", (1,));
    println!("{:#?}", ());
    let empty: Vec<i32> = vec![];
    println!("{:#?}", empty);
    println!("{:#?}", Some(Some(3)));
    let r: Result<Vec<i32>, String> = Ok(vec![1]);
    println!("{:#?}", r);
    let mut hm = HashMap::new();
    hm.insert(1, (2.0, '3'));
    println!("{:#?}", hm);
    println!("{:#?}", [[1, 2], [3, 4]]);
    println!("{:#?}", "str");
    println!("{:#?}", 1.0);
    let mut inner = BTreeMap::new();
    inner.insert(1, vec![Unit]);
    let nested = vec![inner];
    println!("{:#?}", nested);
    let text = format!("{:#?}", vec![Some("x")]);
    println!("{}", text.lines().count());
    println!("{:#?}", std::cell::RefCell::new(1));
}
