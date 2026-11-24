use std::collections::BTreeMap;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Player {
    name: String,
    score: i32,
    level: u32,
}

fn main() {
    let mut players = vec![
        Player { name: "ann".into(), score: 40, level: 1 },
        Player { name: "bob".into(), score: 95, level: 2 },
        Player { name: "cyd".into(), score: 70, level: 1 },
    ];
    for p in players.iter_mut() {
        p.score += 10;
        if p.score > 75 {
            p.level += 1;
        }
    }
    for p in &mut players {
        p.name = p.name.to_uppercase();
    }
    players.iter_mut().filter(|p| p.level == 1).for_each(|p| p.score *= 2);
    if let Some(best) = players.iter_mut().max_by_key(|p| p.score) {
        best.name.push('*');
    }
    for p in &players {
        println!("{} {} {}", p.name, p.score, p.level);
    }

    let text = "the cat and the dog and the bird";
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for w in text.split_whitespace() {
        *counts.entry(w).or_insert(0) += 1;
    }
    println!("{:?}", counts);

    let mut groups: HashMap<u32, Vec<String>> = HashMap::new();
    for p in &players {
        groups.entry(p.level).or_insert_with(Vec::new).push(p.name.clone());
    }
    let mut keys: Vec<_> = groups.keys().copied().collect();
    keys.sort();
    for k in keys {
        println!("level {}: {:?}", k, groups[&k]);
    }

    let mut totals: HashMap<String, i32> = HashMap::new();
    for (name, pts) in [("a", 3), ("b", 4), ("a", 5)] {
        totals.entry(name.to_string()).and_modify(|t| *t += pts).or_insert(pts);
    }
    if let Some(v) = totals.get_mut("b") {
        *v *= 10;
    }
    let mut pairs: Vec<_> = totals.into_iter().collect();
    pairs.sort();
    println!("{:?}", pairs);

    let mut grid = vec![vec![0; 3]; 3];
    for (r, row) in grid.iter_mut().enumerate() {
        for (c, cell) in row.iter_mut().enumerate() {
            *cell = r * 3 + c;
        }
    }
    println!("{:?}", grid);
    let mut nums = [3, 1, 2];
    for n in nums.iter_mut().rev() {
        *n *= 10;
    }
    if let Some(last) = nums.last_mut() {
        *last += 1;
    }
    if let Some(f) = nums.first_mut() {
        *f -= 1;
    }
    println!("{:?}", nums);

    let mut stock: BTreeMap<String, Vec<u32>> = BTreeMap::new();
    for (k, v) in [("b", 1), ("a", 2), ("b", 3)] {
        let e = stock.entry(k.to_string()).or_default();
        e.push(v);
        e.sort_unstable_by(|x, y| y.cmp(x));
    }
    for (_, list) in stock.iter_mut() {
        list.push(0);
    }
    for list in stock.values_mut() {
        list.retain(|&x| x != 1);
    }
    println!("{:?}", stock);
}
