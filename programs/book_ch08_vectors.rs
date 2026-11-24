// Chapter 8.1: storing lists of values with vectors.
#[derive(Debug, Clone, PartialEq)]
enum SpreadsheetCell {
    Int(i32),
    Float(f64),
    Text(String),
}

impl SpreadsheetCell {
    fn render(&self) -> String {
        match self {
            SpreadsheetCell::Int(i) => format!("{i}"),
            SpreadsheetCell::Float(f) => format!("{f:.2}"),
            SpreadsheetCell::Text(s) => format!("'{s}'"),
        }
    }

    fn numeric(&self) -> Option<f64> {
        match *self {
            SpreadsheetCell::Int(i) => Some(i as f64),
            SpreadsheetCell::Float(f) => Some(f),
            SpreadsheetCell::Text(_) => None,
        }
    }
}

fn largest_index(v: &[i32]) -> Option<usize> {
    if v.is_empty() {
        return None;
    }
    let mut best = 0;
    for i in 1..v.len() {
        if v[i] > v[best] {
            best = i;
        }
    }
    Some(best)
}

fn main() {
    let v: Vec<i32> = Vec::new();
    println!("empty: {:?} len {}", v, v.len());

    let v = vec![1, 2, 3];
    println!("{:?}", v);

    let mut v = Vec::new();
    v.push(5);
    v.push(6);
    v.push(7);
    v.push(8);
    println!("{:?}", v);

    let v = vec![1, 2, 3, 4, 5];
    let third: &i32 = &v[2];
    println!("The third element is {third}");
    let third: Option<&i32> = v.get(2);
    match third {
        Some(third) => println!("The third element is {third}"),
        None => println!("There is no third element."),
    }
    match v.get(100) {
        Some(x) => println!("got {x}"),
        None => println!("There is no hundredth element."),
    }

    let mut v = vec![1, 2, 3, 4, 5];
    let first = v[0];
    v.push(6);
    println!("The first element is: {first}, now {:?}", v);

    let v = vec![100, 32, 57];
    for n_ref in &v {
        let n_plus_one: i32 = *n_ref + 1;
        println!("{n_plus_one}");
    }

    let mut v = vec![100, 32, 57];
    for n_ref in &mut v {
        *n_ref += 50;
    }
    println!("{:?}", v);

    let mut v = vec![1, 2, 3];
    for i in 0..v.len() {
        v[i] *= 10;
    }
    let popped = v.pop();
    println!("{:?} popped {:?}", v, popped);
    v.insert(0, 5);
    v.extend([7, 8, 9]);
    let removed = v.remove(1);
    println!("{:?} removed {removed}", v);
    v.retain(|&x| x != 8);
    println!("retain: {:?}", v);
    v.truncate(3);
    println!("truncate: {:?} contains 7? {}", v, v.contains(&7));
    v.reverse();
    println!("reverse: {:?}", v);
    v.sort();
    println!("sort: {:?}", v);
    v.dedup();
    v.swap(0, 2);
    println!("swap: {:?} first {:?} last {:?}", v, v.first(), v.last());

    let row = vec![
        SpreadsheetCell::Int(3),
        SpreadsheetCell::Text(String::from("blue")),
        SpreadsheetCell::Float(10.12),
        SpreadsheetCell::Int(-4),
    ];
    for cell in &row {
        println!("{:?} renders as {}", cell, cell.render());
    }
    let total: f64 = row.iter().filter_map(|c| c.numeric()).sum();
    println!("numeric total = {total:.2}");
    let texts: Vec<&SpreadsheetCell> = row.iter().filter(|c| matches!(c, SpreadsheetCell::Text(_))).collect();
    println!("texts = {:?}", texts);
    let mut row2 = row.clone();
    row2[1] = SpreadsheetCell::Text(String::from("green"));
    println!("same row? {} {}", row == row.clone(), row == row2);

    let scores = vec![4, 19, 7, 19, 2];
    println!("largest index {:?}", largest_index(&scores));
    println!("largest index of empty {:?}", largest_index(&[]));

    let grid = vec![vec![0; 3]; 2];
    println!("{:?}", grid);
    let mut grid = grid;
    grid[1][2] = 9;
    for row in &grid {
        let line: Vec<String> = row.iter().map(|n| n.to_string()).collect();
        println!("{}", line.join(" "));
    }

    let words = vec!["apple", "bob", "cat"];
    let lens: Vec<usize> = words.iter().map(|w| w.len()).collect();
    let (short, long): (Vec<&str>, Vec<&str>) = words.iter().partition(|w| w.len() <= 3);
    println!("{:?} {:?} {:?}", lens, short, long);
    let chunks: Vec<Vec<i32>> = (1..=7).collect::<Vec<_>>().chunks(3).map(|c| c.to_vec()).collect();
    println!("{:?}", chunks);
    let windows: Vec<i32> = [1, 4, 9, 16, 25].windows(2).map(|w| w[1] - w[0]).collect();
    println!("{:?}", windows);
}
