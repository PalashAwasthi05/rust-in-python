// Arrays of non-Copy values: String, Vec, Option<String>, structs with Default, from_fn, map.
#[derive(Debug, Default, Clone, PartialEq)]
struct Slot {
    name: String,
    items: Vec<u32>,
}

fn main() {
    let names: [String; 3] = [String::from("ann"), String::from("bob"), String::from("cy")];
    let upper = names.clone().map(|s| s.to_uppercase());
    println!("{:?} -> {:?}", names, upper);
    let lens = names.each_ref().map(|s| s.len());
    println!("{:?}", lens);

    let squares: [u64; 6] = std::array::from_fn(|i| (i * i) as u64);
    println!("{:?}", squares);
    let labels: [String; 4] = std::array::from_fn(|i| format!("cell{}", i));
    println!("{:?}", labels);
    let grid: [[usize; 3]; 2] = std::array::from_fn(|r| std::array::from_fn(|c| r * 10 + c));
    println!("{:?}", grid);

    let mut slots: [Slot; 3] = Default::default();
    slots[1].name.push_str("middle");
    slots[1].items.push(5);
    slots[2] = Slot { name: "last".to_string(), items: vec![1, 2] };
    println!("{:?}", slots);
    println!("{:#?}", slots[2]);

    let units = [(); 3].map(|_| Vec::<i32>::new());
    println!("{:?} {}", units, units.len());

    let mut opts: [Option<String>; 4] = [None, Some("x".into()), None, Some("y".into())];
    let taken = opts[1].take();
    println!("{:?} {:?}", taken, opts);
    let present: Vec<&String> = opts.iter().flatten().collect();
    println!("{:?}", present);
    opts[0] = Some(String::from("z"));
    println!("{}", opts.iter().filter(|o| o.is_some()).count());

    let empty: [Vec<char>; 2] = Default::default();
    println!("{:?}", empty);

    // Moving out of an array by value with into_iter (2021 edition).
    let owned = [String::from("one"), String::from("two"), String::from("three")];
    let mut total = String::new();
    for s in owned {
        total.push_str(&s);
        total.push('/');
    }
    println!("{}", total);

    let words = ["alpha".to_string(), "beta".to_string()];
    let joined: Vec<String> = words.into_iter().map(|w| w + "!").collect();
    println!("{:?}", joined);

    let nested = [vec![1, 2], vec![3], vec![]];
    let flat: Vec<i32> = nested.into_iter().flatten().collect();
    println!("{:?}", flat);

    let mut arr = [String::from("b"), String::from("c"), String::from("a")];
    arr.sort();
    println!("{:?}", arr);
    arr.swap(0, 2);
    println!("{:?}", arr);
    let moved = std::mem::take(&mut arr[1]);
    println!("{:?} {:?}", moved, arr);
    let replaced = std::mem::replace(&mut arr[0], "new".to_string());
    println!("{} {:?}", replaced, arr);
}
