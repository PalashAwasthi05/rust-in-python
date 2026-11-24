// Day 4 2022 + interval merging.
const INPUT: &str = r"2-4,6-8
2-3,4-5
5-7,7-9
2-8,3-7
6-6,4-6
2-6,4-8
12-20,1-3
15-18,30-40";

fn parse_range(s: &str) -> (u32, u32) {
    let (a, b) = s.split_once('-').unwrap();
    (a.parse().unwrap(), b.parse().unwrap())
}

fn merge(mut ranges: Vec<(u32, u32)>) -> Vec<(u32, u32)> {
    ranges.sort();
    let mut out: Vec<(u32, u32)> = Vec::new();
    for (lo, hi) in ranges {
        if let Some(last) = out.last_mut() {
            if lo <= last.1 + 1 {
                last.1 = last.1.max(hi);
                continue;
            }
        }
        out.push((lo, hi));
    }
    out
}

fn main() {
    let pairs: Vec<((u32, u32), (u32, u32))> = INPUT
        .lines()
        .map(|l| {
            let (a, b) = l.split_once(',').unwrap();
            (parse_range(a), parse_range(b))
        })
        .collect();
    let contained = pairs
        .iter()
        .filter(|((a1, a2), (b1, b2))| (a1 <= b1 && b2 <= a2) || (b1 <= a1 && a2 <= b2))
        .count();
    let overlap = pairs.iter().filter(|(a, b)| a.0 <= b.1 && b.0 <= a.1).count();
    println!("part 1: {contained}");
    println!("part 2: {overlap}");
    let all: Vec<(u32, u32)> = pairs.iter().flat_map(|&(a, b)| [a, b]).collect();
    let merged = merge(all);
    println!("merged: {:?}", merged);
    let covered: u32 = merged.iter().map(|(lo, hi)| hi - lo + 1).sum();
    println!("covered sections: {}", covered);
}
