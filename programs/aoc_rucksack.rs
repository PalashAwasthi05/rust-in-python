// Day 3 2022 style: set intersection via bitmasks.
const INPUT: &str = r"vJrwpWtwJgWrhcsFMMfFFhFp
jqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL
PmmdzqPrVvPwwTWBwg
wMqvLMZHhHMvwLHjbvcjnnSBnvTQFn
ttgJtRGJQctTZtZT
CrZsJsPPZsGzwwsLwLmpwMDw";

fn priority(c: char) -> u32 {
    match c {
        'a'..='z' => c as u32 - 'a' as u32 + 1,
        'A'..='Z' => c as u32 - 'A' as u32 + 27,
        _ => panic!("bad item {c}"),
    }
}

fn mask(s: &str) -> u64 {
    s.chars().fold(0u64, |m, c| m | (1u64 << priority(c)))
}

fn main() {
    let lines: Vec<&str> = INPUT.lines().collect();
    let mut part1 = 0;
    for line in &lines {
        let (a, b) = line.split_at(line.len() / 2);
        let common = mask(a) & mask(b);
        part1 += common.trailing_zeros();
    }
    println!("part 1: {}", part1);
    let part2: u32 = lines
        .chunks(3)
        .map(|g| (mask(g[0]) & mask(g[1]) & mask(g[2])).trailing_zeros())
        .sum();
    println!("part 2: {}", part2);
    let all = lines.iter().fold(0u64, |m, l| m | mask(l));
    println!("distinct priorities: {} (mask {:#x})", all.count_ones(), all);
}
