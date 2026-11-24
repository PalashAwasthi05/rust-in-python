// Maps, sets, derived pretty Debug, and number/text formatting edge cases.
use std::collections::*;
use std::fmt::Write as _;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Color { Red, Green }
#[derive(Debug, Default)]
struct Cfg { name: String, level: u8, tags: Vec<String>, extra: Option<HashMap<String, i32>> }
fn main() {
    let mut m: HashMap<Color, Vec<u32>> = HashMap::new();
    m.entry(Color::Red).or_default().push(1);
    m.entry(Color::Red).or_default().push(2);
    m.entry(Color::Green).or_insert_with(Vec::new).push(3);
    let mut keys: Vec<_> = m.keys().copied().collect();
    keys.sort();
    println!("{:?}", keys);
    println!("{:?}", m.get(&Color::Red).map(Vec::len));
    println!("{:?}", m.values().map(|v| v.len()).sum::<usize>());
    let total: u32 = m.values().flatten().sum();
    println!("{}", total);
    let mut bt: BTreeMap<&str, i32> = BTreeMap::new();
    for (i, w) in "one two three two one one".split(' ').enumerate() {
        bt.entry(w).and_modify(|e| *e += i as i32).or_insert(i as i32);
    }
    println!("{:?}", bt);
    println!("{:?}", bt.iter().max_by_key(|(_, &v)| v));
    println!("{:?}", bt.range("o".."tz").map(|(k, _)| *k).collect::<Vec<_>>());
    println!("{:?}", bt.keys().next_back());
    println!("{:?}", bt.values().rev().collect::<Vec<_>>());
    println!("{:?}", bt.contains_key("two"));
    let removed = bt.remove("two");
    println!("{:?} {}", removed, bt.len());
    let mut hs: HashSet<String> = HashSet::new();
    hs.insert("a".to_string());
    println!("{}", hs.contains("a"));
    let bs: BTreeSet<i32> = (1..10).filter(|x| x % 3 == 0).collect();
    println!("{:?} {:?}", bs, bs.iter().rev().next());
    println!("{:?}", bs.range(4..).next());
    let c = Cfg { name: "x".into(), level: 3, tags: vec!["a".into()], extra: Some(HashMap::from([("k".to_string(), 1)])) };
    println!("{:#?}", c);
    println!("{:?}", Cfg::default());
    let mut out = String::new();
    write!(out, "{:>5}|{:<5}|{:^5}|", 1.5, -2, 'x').unwrap();
    writeln!(out, "{:05.1}", 2.25).unwrap();
    print!("{}", out);
    println!("{:?}", format!("{:10.3}|{:<10.3e}|", 3.14159, 31415.9));
    println!("{:?}", format!("{:+}|{:+}|{:+.1}", 0, -0.0, 0.05));
    println!("{}", format!("{:>1$}", "ab", 5) + &format!("{:.*}", 1, 9.99));
    println!("{}", format!("{:#010b}|{:#o}|{:#X}", 5u8, 64, 48879));
    println!("{:?} {:?}", i64::MIN, (i8::MIN as i16).abs());
    println!("{:e} {:e} {:E}", 0.001, 123456789u64, 1e-10);
    println!("{} {} {}", 1e15, 1e16, 1.5e300);
    println!("{} {}", f32::MAX, f32::MIN_POSITIVE);
    println!("{} {:?}", 16777217.0f32, 0.1f32 + 0.2f32);
    println!("{}", 255u8 as i8 as i32);
    println!("{}", (3.7f64 as i64, -3.7f64 as i64, 1e300f64 as f32).2);
    println!("{:?}", "Mary had a little lamb".split_whitespace().map(|w| w.len()).collect::<Vec<_>>());
    println!("{:?}", "a,b,,c".split(',').filter(|s| !s.is_empty()).collect::<Vec<_>>());
    println!("{:?}", "  x  ".trim_start().len());
    println!("{:?}", "abc".bytes().rev().map(|b| b as char).collect::<String>());
    println!("{:?}", "hello"[1..].starts_with("ell"));
    println!("{:?}", "Hello World".chars().filter(|c| c.is_uppercase()).collect::<Vec<_>>());
    println!("{:?}", "über".to_uppercase());
    println!("{:?}", "ß".to_uppercase());
    println!("{:?}", "İ".to_lowercase().len());
    println!("{:?}", 'ß'.to_uppercase().collect::<String>());
    println!("{:?}", "tschüß".len());
    println!("{:?}", "x".cmp("y"));
    println!("{:?}", "abc".to_owned() + "def");
    println!("{:?}", String::from("abc").into_bytes());
    println!("{:?}", "a-b_c".replace(|c: char| !c.is_alphanumeric(), " "));
    println!("{:?}", "a1b2".chars().partition::<String, _>(|c| c.is_numeric()));
    println!("{:?}", "rust".chars().map(|c| ((c as u8) + 1) as char).collect::<String>());
    println!("{:?}", "one two".split(' ').map(String::from).collect::<Vec<String>>());
    println!("{:?}", ["x", "y"].iter().map(|s| s.to_uppercase()).collect::<Vec<_>>().concat());
    println!("{:?}", "abcdef".char_indices().filter(|(i, _)| i % 2 == 0).map(|(_, c)| c).collect::<String>());
    let mut letters: Vec<char> = "hello".chars().collect();
    letters.sort_unstable_by(|a, b| b.cmp(a));
    println!("{:?}", letters.into_iter().collect::<String>());
    println!("{:?}", "3 4 5".split(' ').map(|x| x.parse::<f64>().unwrap()).fold(0.0, |a, b| a + b));
    println!("{:?}", "10".parse::<u8>().ok().and_then(|x| x.checked_mul(30)));
}
