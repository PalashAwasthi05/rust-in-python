// HashMap and BTreeMap APIs.
use std::collections::{BTreeMap, HashMap};

fn word_freq(text: &str) -> BTreeMap<String, usize> {
    let mut freq = BTreeMap::new();
    for w in text.split_whitespace() {
        let w = w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase();
        if w.is_empty() {
            continue;
        }
        *freq.entry(w).or_insert(0) += 1;
    }
    freq
}

fn main() {
    let freq = word_freq("The cat and the hat. The END, the end!");
    println!("{:?}", freq);
    let mut by_count: Vec<(&String, &usize)> = freq.iter().collect();
    by_count.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    println!("top: {:?}", &by_count[..2]);

    let mut stock: HashMap<&str, i32> = HashMap::from([("apple", 3), ("pear", 0), ("plum", 7)]);
    stock.entry("apple").and_modify(|n| *n += 10).or_insert(1);
    stock.entry("kiwi").and_modify(|n| *n += 10).or_insert(1);
    *stock.entry("plum").or_default() -= 2;
    stock.entry("fig").or_insert_with(|| 40 + 2);
    let key_len = stock.entry("melon").or_insert_with_key(|k| k.len() as i32);
    *key_len *= 2;
    for v in stock.values_mut() {
        *v *= 2;
    }
    for (_, v) in stock.iter_mut() {
        *v += 1;
    }
    stock.retain(|_, v| *v > 1);
    let mut items: Vec<_> = stock.iter().collect();
    items.sort();
    println!("stock: {:?}", items);
    println!("get apple {:?} missing {:?}", stock.get("apple"), stock.get("grape"));
    println!("contains plum {} len {}", stock.contains_key("plum"), stock.len());
    if let Some(v) = stock.get_mut("fig") {
        *v = 0;
    }
    println!("remove fig {:?} again {:?}", stock.remove("fig"), stock.remove("fig"));
    println!("insert returns {:?} {:?}", stock.insert("new", 1), stock.insert("new", 2));
    let mut keys: Vec<&&str> = stock.keys().collect();
    keys.sort();
    println!("keys {:?}", keys);
    let total: i32 = stock.values().sum();
    println!("total {}", total);

    let mut scores: BTreeMap<u32, &str> = BTreeMap::new();
    scores.extend([(90, "ann"), (75, "bob"), (82, "cy"), (60, "dee"), (99, "eve")]);
    println!("first {:?} last {:?}", scores.first_key_value(), scores.last_key_value());
    let middle: Vec<_> = scores.range(70..90).collect();
    println!("range 70..90 {:?}", middle);
    let upper: Vec<&str> = scores.range(80..).map(|(_, n)| *n).collect();
    println!("range 80.. {:?}", upper);
    let inclusive: Vec<u32> = scores.range(..=82).map(|(k, _)| *k).collect();
    println!("range ..=82 {:?}", inclusive);
    println!("pop_first {:?} pop_last {:?}", scores.pop_first(), scores.pop_last());
    println!("rev {:?}", scores.iter().rev().map(|(k, _)| k).collect::<Vec<_>>());
    println!("into_values {:?}", scores.clone().into_values().collect::<Vec<_>>());
    println!("into_keys {:?}", scores.clone().into_keys().collect::<Vec<_>>());
    let index = &scores[&82];
    println!("index {}", index);
    let mut grouped: BTreeMap<usize, Vec<&str>> = BTreeMap::new();
    for w in ["a", "bb", "cc", "d", "eee"] {
        grouped.entry(w.len()).or_insert_with(Vec::new).push(w);
    }
    println!("grouped {:?}", grouped);
    let inverted: BTreeMap<&str, u32> = scores.iter().map(|(k, v)| (*v, *k)).collect();
    println!("inverted {:?}", inverted);
    let mut counts: HashMap<char, usize> = HashMap::new();
    for ch in "mississippi".chars() {
        *counts.entry(ch).or_insert(0) += 1;
    }
    let mut cv: Vec<(char, usize)> = counts.into_iter().collect();
    cv.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    println!("counts {:?}", cv);
    let m: HashMap<i32, i32> = (1..4).map(|i| (i, i * i)).collect();
    let mut mv: Vec<_> = m.into_values().collect();
    mv.sort();
    println!("into_values {:?}", mv);
    let mut bt = BTreeMap::from([(3, 'c'), (1, 'a')]);
    if let Some(v) = bt.get_mut(&1) {
        *v = 'A';
    }
    bt.append(&mut BTreeMap::from([(2, 'b')]));
    println!("bt {:?} empty {}", bt, bt.is_empty());
    let mut e = BTreeMap::<String, Vec<i32>>::new();
    e.entry("x".into()).or_default().push(1);
    e.entry("x".into()).or_default().push(2);
    println!("{:?}", e);
}
