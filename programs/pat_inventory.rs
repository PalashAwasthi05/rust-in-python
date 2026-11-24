// Inventory system: struct-variant let-else, match ergonomics on nested refs, consts in patterns.
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
enum Item {
    Tool { name: String, durability: u8 },
    Food { name: String, calories: u32 },
    Coin(u32),
    Bundle(Vec<Item>),
}

struct Limits;
impl Limits {
    const MAX_DURABILITY: u8 = 100;
    const BROKEN: u8 = 0;
}

const FEAST: u32 = 1000;

fn describe(item: &Item) -> String {
    match item {
        Item::Tool { durability: Limits::BROKEN, name } => format!("broken {}", name),
        Item::Tool { durability: Limits::MAX_DURABILITY, name } => format!("pristine {}", name),
        Item::Tool { name, durability: d @ 1..=20 } => format!("worn {} ({}%)", name, d),
        Item::Tool { name, .. } => format!("{}", name),
        Item::Food { calories: FEAST.., name } => format!("feast of {}", name),
        Item::Food { name, calories } => format!("{} ({} kcal)", name, calories),
        Item::Coin(0) => "nothing".into(),
        Item::Coin(n @ (1 | 2)) => format!("{} small coin{}", n, if *n == 1 { "" } else { "s" }),
        Item::Coin(n) => format!("{} coins", n),
        Item::Bundle(items) if items.is_empty() => "empty bundle".into(),
        Item::Bundle(items) => match items.as_slice() {
            [only] => format!("bundle of just {}", describe(only)),
            [first, .., last] => format!("bundle {}..{} [{}]", describe(first), describe(last), items.len()),
            [] => unreachable!(),
        },
    }
}

fn total_coins(items: &[Item]) -> u32 {
    items.iter().map(|it| match it {
        Item::Coin(n) => *n,
        Item::Bundle(inner) => total_coins(inner),
        _ => 0,
    }).sum()
}

fn food_name(item: &Item) -> Option<&str> {
    let Item::Food { name, .. } = item else { return None };
    Some(name.as_str())
}

fn parse_item(line: &str) -> Result<Item, String> {
    let parse_num = |s: &str| -> Result<u32, String> {
        s.trim().parse::<u32>().map_err(|e| format!("{}: {}", s.trim(), e))
    };
    let parts: Vec<&str> = line.split(':').collect();
    let item = match parts[..] {
        ["tool", name, dur] => {
            let d = parse_num(dur)?;
            let durability = u8::try_from(d).map_err(|_| format!("durability {} too big", d))?;
            Item::Tool { name: name.to_string(), durability }
        }
        ["food", name, cal] => Item::Food { name: name.into(), calories: parse_num(cal)? },
        ["coin", n] => Item::Coin(parse_num(n)?),
        [kind, ..] => return Err(format!("unknown kind {:?}", kind)),
        [] => return Err("empty".into()),
    };
    Ok(item)
}

fn main() {
    let lines = ["tool:axe:0", "tool:sword:100", "tool:pick:15", "tool:hoe:50", "food:pie:1200",
                 "food:apple:95", "coin:0", "coin:1", "coin:2", "coin:40", "tool:x:300", "gem:ruby", "coin:abc"];
    let mut items = Vec::new();
    for line in lines {
        match parse_item(line) {
            Ok(item) => items.push(item),
            Err(e) => println!("skip {:?}: {}", line, e),
        }
    }
    items.push(Item::Bundle(vec![]));
    items.push(Item::Bundle(vec![Item::Coin(5)]));
    items.push(Item::Bundle(vec![Item::Coin(5), Item::Food { name: "bread".into(), calories: 300 }, Item::Coin(7)]));
    for item in &items {
        println!("{}", describe(item));
    }
    println!("coins: {}", total_coins(&items));
    let foods: Vec<&str> = items.iter().filter_map(food_name).collect();
    println!("foods: {:?}", foods);
    let mut by_kind: BTreeMap<&str, usize> = BTreeMap::new();
    for item in &items {
        let kind = match item { Item::Tool { .. } => "tool", Item::Food { .. } => "food", Item::Coin(_) => "coin", Item::Bundle(_) => "bundle" };
        *by_kind.entry(kind).or_default() += 1;
    }
    println!("{:?}", by_kind);
    let nested = &Some(&(1, Some("x")));
    match nested {
        Some((n, Some(s))) => println!("deep {} {}", n, s),
        Some((_, None)) | None => println!("shallow"),
    }
    let worn = items.iter().filter(|i| matches!(i, Item::Tool { durability, .. } if *durability < 60)).count();
    println!("worn tools {}", worn);
    let adder = |a: i32| move |b: i32| move |c: i32| a + b + c;
    println!("{}", adder(1)(2)(3));
    let checked_sum = |xs: &[&str]| -> Option<i32> {
        let mut acc = 0i32;
        for x in xs {
            acc = acc.checked_add(x.parse::<i32>().ok()?)?;
        }
        Some(acc)
    };
    println!("{:?} {:?} {:?}", checked_sum(&["1", "2"]), checked_sum(&["1", "z"]), checked_sum(&["2147483647", "1"]));
}
