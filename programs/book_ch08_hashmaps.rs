// Chapter 8.3: storing keys with associated values in hash maps, plus the chapter exercises.
use std::collections::BTreeMap;
use std::collections::HashMap;

fn mean_median_mode(values: &[i32]) -> (f64, f64, i32) {
    let sum: i32 = values.iter().sum();
    let mean = sum as f64 / values.len() as f64;
    let mut sorted = values.to_vec();
    sorted.sort();
    let mid = sorted.len() / 2;
    let median = if sorted.len() % 2 == 0 {
        (sorted[mid - 1] + sorted[mid]) as f64 / 2.0
    } else {
        sorted[mid] as f64
    };
    let mut counts = HashMap::new();
    for &v in values {
        *counts.entry(v).or_insert(0) += 1;
    }
    let mut mode = sorted[0];
    let mut best = 0;
    for &v in &sorted {
        let c = counts[&v];
        if c > best {
            best = c;
            mode = v;
        }
    }
    (mean, median, mode)
}

fn main() {
    let mut scores = HashMap::new();
    scores.insert(String::from("Blue"), 10);
    scores.insert(String::from("Yellow"), 50);

    let team_name = String::from("Blue");
    let score = scores.get(&team_name).copied().unwrap_or(0);
    println!("Blue: {score}");
    println!("Red: {}", scores.get("Red").copied().unwrap_or(0));

    let mut pairs: Vec<(&String, &i32)> = scores.iter().collect();
    pairs.sort();
    for (key, value) in pairs {
        println!("{key}: {value}");
    }

    let field_name = String::from("Favorite color");
    let field_value = String::from("Blue");
    let mut map = HashMap::new();
    map.insert(field_name, field_value);
    println!("{:?}", map);

    scores.insert(String::from("Blue"), 25);
    println!("Blue overwritten: {}", scores["Blue"]);

    scores.entry(String::from("Yellow")).or_insert(50);
    scores.entry(String::from("Green")).or_insert(5);
    let mut sorted: Vec<_> = scores.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    println!("{:?}", sorted);

    let text = "hello world wonderful world";
    let mut map = HashMap::new();
    for word in text.split_whitespace() {
        let count = map.entry(word).or_insert(0);
        *count += 1;
    }
    let mut counts: Vec<_> = map.iter().collect();
    counts.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    println!("{:?}", counts);

    let teams = vec![String::from("Blue"), String::from("Yellow")];
    let initial_scores = vec![10, 50];
    let zipped: HashMap<_, _> = teams.iter().zip(initial_scores.iter()).collect();
    let mut keys: Vec<_> = zipped.keys().collect();
    keys.sort();
    println!("{:?} total {}", keys, zipped.values().copied().sum::<i32>());

    let (mean, median, mode) = mean_median_mode(&[3, 7, 7, 2, 9, 4, 7, 1, 2]);
    println!("mean {mean:.3} median {median} mode {mode}");
    let (mean, median, mode) = mean_median_mode(&[1, 2, 3, 4]);
    println!("mean {mean} median {median} mode {mode}");

    // "Add Sally to Engineering" style company directory.
    let commands = [
        "Add Sally to Engineering",
        "Add Amir to Sales",
        "Add Bob to Engineering",
        "Add Zoe to Sales",
        "Add Carl to Marketing",
        "Remove Bob",
        "Add Ann to Engineering",
    ];
    let mut company: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for command in commands {
        let parts: Vec<&str> = command.split(' ').collect();
        match parts.as_slice() {
            ["Add", name, "to", dept] => {
                company.entry(dept.to_string()).or_default().push(name.to_string());
            }
            ["Remove", name] => {
                for people in company.values_mut() {
                    people.retain(|p| p != name);
                }
            }
            _ => println!("bad command: {command}"),
        }
    }
    for (dept, people) in company.iter_mut() {
        people.sort();
        println!("{dept}: {}", people.join(", "));
    }
    let everyone: Vec<&String> = {
        let mut all: Vec<&String> = company.values().flatten().collect();
        all.sort();
        all
    };
    println!("everyone: {:?}", everyone);

    let mut letters: HashMap<char, usize> = HashMap::new();
    for ch in "mississippi".chars() {
        *letters.entry(ch).or_default() += 1;
    }
    let mut letters: Vec<_> = letters.into_iter().collect();
    letters.sort();
    println!("{:?}", letters);
    let mut inventory: HashMap<&str, i32> = HashMap::new();
    inventory.insert("apples", 3);
    if let Some(n) = inventory.get_mut("apples") {
        *n += 2;
    }
    inventory.entry("pears").and_modify(|n| *n += 1).or_insert(1);
    inventory.entry("pears").and_modify(|n| *n += 1).or_insert(1);
    println!("apples {} pears {} has plums {}", inventory["apples"], inventory["pears"], inventory.contains_key("plums"));
    println!("removed {:?} len {}", inventory.remove("apples"), inventory.len());
}
