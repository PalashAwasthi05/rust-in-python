// Day 12 2023: counting arrangements with memoized recursion over slices.
use std::collections::HashMap;

const INPUT: &str = r"???.### 1,1,3
.??..??...?##. 1,1,3
?#?#?#?#?#?#?#? 1,3,1,6
????.#...#... 4,1,1
????.######..#####. 1,6,5
?###???????? 3,2,1";

struct Solver<'a> {
    springs: &'a [u8],
    groups: &'a [usize],
    memo: HashMap<(usize, usize), u64>,
}

impl<'a> Solver<'a> {
    fn new(springs: &'a [u8], groups: &'a [usize]) -> Self {
        Solver { springs, groups, memo: HashMap::new() }
    }

    fn count(&mut self, i: usize, g: usize) -> u64 {
        if let Some(&v) = self.memo.get(&(i, g)) {
            return v;
        }
        let result = if i >= self.springs.len() {
            if g == self.groups.len() { 1 } else { 0 }
        } else {
            let mut total = 0;
            let c = self.springs[i];
            if c == b'.' || c == b'?' {
                total += self.count(i + 1, g);
            }
            if (c == b'#' || c == b'?') && g < self.groups.len() {
                let len = self.groups[g];
                let end = i + len;
                if end <= self.springs.len()
                    && self.springs[i..end].iter().all(|&b| b != b'.')
                    && self.springs.get(end) != Some(&b'#')
                {
                    total += self.count(end + 1, g + 1);
                }
            }
            total
        };
        self.memo.insert((i, g), result);
        result
    }
}

fn arrangements(springs: &str, groups: &[usize]) -> u64 {
    let mut solver = Solver::new(springs.as_bytes(), groups);
    solver.count(0, 0)
}

fn main() {
    let rows: Vec<(String, Vec<usize>)> = INPUT
        .lines()
        .map(|l| {
            let (s, g) = l.split_once(' ').unwrap();
            (s.to_owned(), g.split(',').map(|n| n.parse().unwrap()).collect())
        })
        .collect();
    let mut part1 = 0;
    for (s, g) in &rows {
        let n = arrangements(s, g);
        println!("{s:>20} {g:?} -> {n}");
        part1 += n;
    }
    println!("part 1: {part1}");
    let part2: u64 = rows
        .iter()
        .map(|(s, g)| {
            let unfolded = vec![s.as_str(); 5].join("?");
            let groups = g.repeat(5);
            arrangements(&unfolded, &groups)
        })
        .sum();
    println!("part 2: {part2}");
}
