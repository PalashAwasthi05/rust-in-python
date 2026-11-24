// Day 5 2022: crate stacks with moves.
const INPUT: &str = r"    [D]
[N] [C]
[Z] [M] [P]
 1   2   3

move 1 from 2 to 1
move 3 from 1 to 3
move 2 from 2 to 1
move 1 from 1 to 2";

fn parse(input: &str) -> (Vec<Vec<char>>, Vec<(usize, usize, usize)>) {
    let (drawing, moves) = input.split_once("\n\n").unwrap();
    let mut rows: Vec<&str> = drawing.lines().collect();
    let labels = rows.pop().unwrap();
    let n = labels.split_whitespace().count();
    let mut stacks = vec![Vec::new(); n];
    for row in rows.iter().rev() {
        for (i, c) in row.chars().skip(1).step_by(4).enumerate() {
            if c != ' ' {
                stacks[i].push(c);
            }
        }
    }
    let moves = moves
        .lines()
        .map(|l| {
            let nums: Vec<usize> = l.split(' ').filter_map(|w| w.parse().ok()).collect();
            (nums[0], nums[1] - 1, nums[2] - 1)
        })
        .collect();
    (stacks, moves)
}

fn tops(stacks: &[Vec<char>]) -> String {
    stacks.iter().filter_map(|s| s.last()).collect()
}

fn main() {
    let (start, moves) = parse(INPUT);
    let mut a = start.clone();
    for &(count, from, to) in &moves {
        for _ in 0..count {
            let c = a[from].pop().unwrap();
            a[to].push(c);
        }
    }
    println!("part 1: {}", tops(&a));
    let mut b = start;
    for &(count, from, to) in &moves {
        let at = b[from].len() - count;
        let moved: Vec<char> = b[from].drain(at..).collect();
        b[to].extend(moved);
    }
    println!("part 2: {}", tops(&b));
    for (i, s) in b.iter().enumerate() {
        println!("{}: {}", i + 1, s.iter().collect::<String>());
    }
}
