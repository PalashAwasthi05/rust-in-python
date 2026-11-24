// Day 1 2023 style: digits and spelled-out digits.
const INPUT: &str = r"two1nine
eightwothree
abcone2threexyz
xtwone3four
4nineeightseven2
zoneight234
7pqrstsixteen
1abc2
pqr3stu8vwx
treb7uchet";

const WORDS: [&str; 9] = ["one", "two", "three", "four", "five", "six", "seven", "eight", "nine"];

fn digit_at(line: &str, i: usize, words: bool) -> Option<u32> {
    let c = line.as_bytes()[i];
    if c.is_ascii_digit() {
        return Some((c - b'0') as u32);
    }
    if words {
        for (n, w) in WORDS.iter().enumerate() {
            if line[i..].starts_with(w) {
                return Some(n as u32 + 1);
            }
        }
    }
    None
}

fn calibrate(line: &str, words: bool) -> u32 {
    let first = (0..line.len()).find_map(|i| digit_at(line, i, words));
    let last = (0..line.len()).rev().find_map(|i| digit_at(line, i, words));
    match (first, last) {
        (Some(a), Some(b)) => a * 10 + b,
        _ => 0,
    }
}

fn main() {
    let p1: u32 = INPUT.lines().map(|l| calibrate(l, false)).sum();
    let p2: u32 = INPUT.lines().map(|l| calibrate(l, true)).sum();
    println!("part 1: {p1}");
    println!("part 2: {p2}");
    for line in INPUT.lines().take(3) {
        let digits: String = line.chars().filter(|c| c.is_ascii_digit()).collect();
        println!("{:<20} -> {:?} / {}", line, digits, calibrate(line, true));
    }
}
