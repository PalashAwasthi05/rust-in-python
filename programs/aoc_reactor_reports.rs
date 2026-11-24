// Day 2 2024 (windows, all/any) + Day 3 2024 (mul scanner with do/don't).
const REPORTS: &str = r"7 6 4 2 1
1 2 7 8 9
9 7 6 2 1
1 3 2 4 5
8 6 4 4 1
1 3 6 7 9";

const MEMORY: &str = r"xmul(2,4)&mul[3,7]!^don't()_mul(5,5)+mul(32,64](mul(11,8)undo()?mul(8,5))mul(123,4567)";

fn safe(levels: &[i32]) -> bool {
    let diffs: Vec<i32> = levels.windows(2).map(|w| w[1] - w[0]).collect();
    diffs.iter().all(|d| (1..=3).contains(d)) || diffs.iter().all(|d| (-3..=-1).contains(d))
}

fn dampened(levels: &[i32]) -> bool {
    safe(levels)
        || (0..levels.len()).any(|skip| {
            let rest: Vec<i32> = levels
                .iter()
                .enumerate()
                .filter(|&(i, _)| i != skip)
                .map(|(_, &v)| v)
                .collect();
            safe(&rest)
        })
}

fn parse_num(s: &str, stop: char) -> Option<(i64, &str)> {
    let end = s.find(stop)?;
    let digits = &s[..end];
    if digits.is_empty() || digits.len() > 3 || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some((digits.parse().ok()?, &s[end + 1..]))
}

fn scan(mem: &str, conditional: bool) -> i64 {
    let mut total = 0;
    let mut enabled = true;
    for (i, _) in mem.char_indices() {
        let rest = &mem[i..];
        if rest.starts_with("do()") {
            enabled = true;
        } else if rest.starts_with("don't()") {
            enabled = false;
        } else if let Some(args) = rest.strip_prefix("mul(") {
            if let Some((a, after)) = parse_num(args, ',') {
                if let Some((b, _)) = parse_num(after, ')') {
                    if enabled || !conditional {
                        total += a * b;
                    }
                }
            }
        }
    }
    total
}

fn main() {
    let reports: Vec<Vec<i32>> = REPORTS
        .lines()
        .map(|l| l.split_ascii_whitespace().map(|n| n.parse().unwrap()).collect())
        .collect();
    println!("part 1: {}", reports.iter().filter(|r| safe(r)).count());
    println!("part 2: {}", reports.iter().filter(|r| dampened(r)).count());
    println!("mul: {}", scan(MEMORY, false));
    println!("mul with do: {}", scan(MEMORY, true));
    let matches: Vec<(usize, &str)> = MEMORY.match_indices("mul(").collect();
    println!("{} candidates, first at {}", matches.len(), matches[0].0);
}
