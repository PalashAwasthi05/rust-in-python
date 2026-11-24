// Day 1 style: groups of numbers separated by blank lines.
const INPUT: &str = r#"1000
2000
3000

4000

5000
6000

7000
8000
9000

10000
"#;

fn main() {
    let mut elves: Vec<u32> = INPUT
        .trim()
        .split("\n\n")
        .map(|group| group.lines().map(|l| l.trim().parse::<u32>().unwrap()).sum())
        .collect();
    let best = *elves.iter().max().unwrap();
    println!("part 1: {}", best);
    elves.sort_unstable_by(|a, b| b.cmp(a));
    let top3: u32 = elves.iter().take(3).sum();
    println!("part 2: {}", top3);
    let idx = elves.iter().position(|&c| c == best).unwrap();
    println!("richest elf sorted index {idx}, total elves {}", elves.len());
}
