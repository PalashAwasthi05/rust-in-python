use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Coord {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Key {
    Id(u32),
    Name(String),
    Both(u32, String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Rgb(u8, u8, u8);

fn main() {
    let mut visits: HashMap<Coord, u32> = HashMap::new();
    let path = [(0, 0), (1, 0), (1, 1), (0, 0), (1, 1), (0, 0)];
    for (x, y) in path.iter() {
        *visits.entry(Coord { x: *x, y: *y }).or_insert(0) += 1;
    }
    let mut v: Vec<_> = visits.iter().collect();
    v.sort();
    println!("{:?}", v);
    println!("origin: {:?}", visits.get(&Coord { x: 0, y: 0 }));
    println!("missing: {:?}", visits.get(&Coord { x: 9, y: 9 }));

    let mut counts: HashMap<Suit, Vec<u8>> = HashMap::new();
    let deal = [(Suit::Hearts, 3), (Suit::Spades, 12), (Suit::Hearts, 1), (Suit::Clubs, 7), (Suit::Spades, 2)];
    for (s, r) in deal.iter() {
        counts.entry(*s).or_default().push(*r);
    }
    let mut suits: Vec<_> = counts.keys().copied().collect();
    suits.sort();
    for s in &suits {
        println!("{:?}: {:?}", s, counts[s]);
    }

    let mut lookup: HashMap<Key, &str> = HashMap::new();
    lookup.insert(Key::Id(7), "seven");
    lookup.insert(Key::Name("x".to_string()), "ex");
    lookup.insert(Key::Both(7, "x".to_string()), "both");
    lookup.insert(Key::Id(7), "SEVEN");
    println!("len {}", lookup.len());
    println!("{:?}", lookup.get(&Key::Id(7)));
    println!("{:?}", lookup.get(&Key::Both(7, "x".to_string())));
    println!("{:?}", lookup.get(&Key::Both(7, "y".to_string())));
    let mut entries: Vec<(&Key, &&str)> = lookup.iter().collect();
    entries.sort();
    println!("{:?}", entries);

    let mut pairs: HashMap<(i32, i32), &str> = HashMap::new();
    pairs.insert((1, 2), "a");
    pairs.insert((2, 1), "b");
    pairs.insert((1, 2), "c");
    let mut keys: Vec<_> = pairs.keys().collect();
    keys.sort();
    println!("{:?} {:?}", keys, pairs[&(1, 2)]);

    let mut ips: HashMap<[u8; 4], String> = HashMap::new();
    ips.insert([10, 0, 0, 1], "router".to_string());
    ips.insert([192, 168, 1, 20], "laptop".to_string());
    ips.insert([10, 0, 0, 1], "gateway".to_string());
    let mut ip_list: Vec<_> = ips.iter().map(|(k, v)| (*k, v.clone())).collect();
    ip_list.sort();
    for (ip, name) in &ip_list {
        println!("{}.{}.{}.{} {}", ip[0], ip[1], ip[2], ip[3], name);
    }

    let mut by_vec: HashMap<Vec<char>, usize> = HashMap::new();
    for w in ["listen", "silent", "enlist", "google", "gogole", "cat"].iter() {
        let mut k: Vec<char> = w.chars().collect();
        k.sort();
        *by_vec.entry(k).or_insert(0) += 1;
    }
    let mut groups: Vec<(String, usize)> = by_vec.into_iter().map(|(k, v)| (k.into_iter().collect(), v)).collect();
    groups.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    println!("{:?}", groups);

    let mut words: HashMap<String, i32> = HashMap::new();
    for w in "the cat the dog the end dog".split_whitespace() {
        *words.entry(w.to_string()).or_insert(0) += 1;
    }
    let mut wl: Vec<_> = words.into_iter().collect();
    wl.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    println!("{:?}", wl);

    let mut colors: HashMap<Rgb, &str> = HashMap::new();
    colors.insert(Rgb(255, 0, 0), "red");
    colors.insert(Rgb(0, 255, 0), "green");
    println!("{:?} {}", colors.get(&Rgb(255, 0, 0)), colors.contains_key(&Rgb(0, 0, 255)));
    let mut tuple_key: HashMap<(String, Suit), bool> = HashMap::new();
    tuple_key.insert(("a".to_string(), Suit::Diamonds), true);
    println!("{:?}", tuple_key.get(&("a".to_string(), Suit::Diamonds)));
    println!("{:?}", tuple_key.get(&("a".to_string(), Suit::Hearts)));
}
