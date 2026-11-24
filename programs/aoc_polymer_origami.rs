// Day 14 2021 (pair insertion counts) + Day 13 2021 (folding a dot sheet).
use std::collections::{BTreeSet, HashMap};

const POLYMER: &str = r"NNCB

CH -> B
HH -> N
CB -> H
NH -> C
HB -> C
HC -> B
HN -> C
NN -> C
BH -> H
NC -> B
NB -> B
BN -> B
BB -> N
BC -> B
CC -> N
CN -> C";

const SHEET: &str = r"6,10
0,14
9,10
0,3
10,4
4,11
6,0
6,12
4,1
0,13
10,12
3,4
3,0
8,4
1,10
2,14
8,10
9,0

fold along y=7
fold along x=5";

fn polymer(steps: usize) -> u64 {
    let (template, rules) = POLYMER.split_once("\n\n").unwrap();
    let rules: HashMap<(char, char), char> = rules
        .lines()
        .map(|l| {
            let (pair, ins) = l.split_once(" -> ").unwrap();
            let mut cs = pair.chars();
            ((cs.next().unwrap(), cs.next().unwrap()), ins.chars().next().unwrap())
        })
        .collect();
    let chars: Vec<char> = template.chars().collect();
    let mut pairs: HashMap<(char, char), u64> = HashMap::new();
    for w in chars.windows(2) {
        *pairs.entry((w[0], w[1])).or_insert(0) += 1;
    }
    for _ in 0..steps {
        let mut next = HashMap::new();
        for (&(a, b), &n) in &pairs {
            match rules.get(&(a, b)) {
                Some(&c) => {
                    *next.entry((a, c)).or_insert(0) += n;
                    *next.entry((c, b)).or_insert(0) += n;
                }
                None => *next.entry((a, b)).or_insert(0) += n,
            }
        }
        pairs = next;
    }
    let mut counts: HashMap<char, u64> = HashMap::new();
    for (&(a, _), &n) in &pairs {
        *counts.entry(a).or_insert(0) += n;
    }
    *counts.entry(*chars.last().unwrap()).or_insert(0) += 1;
    let mut sorted: Vec<(char, u64)> = counts.into_iter().collect();
    sorted.sort_by_key(|&(c, n)| (std::cmp::Reverse(n), c));
    println!("{steps} steps: {:?}", sorted);
    sorted.first().unwrap().1 - sorted.last().unwrap().1
}

fn fold(dots: &BTreeSet<(u32, u32)>, axis: char, line: u32) -> BTreeSet<(u32, u32)> {
    dots.iter()
        .map(|&(x, y)| match axis {
            'x' if x > line => (2 * line - x, y),
            'y' if y > line => (x, 2 * line - y),
            _ => (x, y),
        })
        .collect()
}

fn main() {
    println!("polymer 10: {}", polymer(10));
    println!("polymer 40: {}", polymer(40));
    let (dots, folds) = SHEET.split_once("\n\n").unwrap();
    let mut dots: BTreeSet<(u32, u32)> = dots
        .lines()
        .filter_map(|l| l.split_once(','))
        .map(|(x, y)| (x.parse().unwrap(), y.parse().unwrap()))
        .collect();
    for (i, f) in folds.lines().enumerate() {
        let (axis, n) = f.trim_start_matches("fold along ").split_once('=').unwrap();
        dots = fold(&dots, axis.chars().next().unwrap(), n.parse().unwrap());
        println!("after fold {}: {} dots", i + 1, dots.len());
    }
    let w = dots.iter().map(|d| d.0).max().unwrap() + 1;
    let h = dots.iter().map(|d| d.1).max().unwrap() + 1;
    let mut canvas = vec![vec![' '; w as usize]; h as usize];
    for &(x, y) in &dots {
        canvas[y as usize][x as usize] = '#';
    }
    for row in canvas {
        println!("|{}|", row.into_iter().collect::<String>());
    }
}
