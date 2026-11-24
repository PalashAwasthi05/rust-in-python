// Day 3 2023: numbers adjacent to symbols, scanning with char_indices.
use std::collections::HashMap;

const INPUT: &str = r"467..114..
...*......
..35..633.
......#...
617*......
.....+.58.
..592.....
......755.
...$.*....
.664.598..";

fn main() {
    let grid: Vec<Vec<char>> = INPUT.lines().map(|l| l.chars().collect()).collect();
    let h = grid.len() as isize;
    let w = grid[0].len() as isize;
    let mut part1 = 0u32;
    let mut gears: HashMap<(usize, usize), Vec<u32>> = HashMap::new();
    for (r, line) in INPUT.lines().enumerate() {
        let mut chars = line.char_indices().peekable();
        while let Some((start, ch)) = chars.next() {
            if !ch.is_ascii_digit() {
                continue;
            }
            let mut end = start;
            while let Some(&(i, c)) = chars.peek() {
                if !c.is_ascii_digit() {
                    break;
                }
                end = i;
                chars.next();
            }
            let value: u32 = line[start..=end].parse().unwrap();
            let mut adjacent = false;
            for rr in (r as isize - 1)..=(r as isize + 1) {
                for cc in (start as isize - 1)..=(end as isize + 1) {
                    if rr < 0 || cc < 0 || rr >= h || cc >= w {
                        continue;
                    }
                    let s = grid[rr as usize][cc as usize];
                    if s != '.' && !s.is_ascii_digit() {
                        adjacent = true;
                        if s == '*' {
                            gears.entry((rr as usize, cc as usize)).or_default().push(value);
                        }
                    }
                }
            }
            if adjacent {
                part1 += value;
            }
        }
    }
    println!("part 1: {part1}");
    let mut keys: Vec<_> = gears.keys().collect();
    keys.sort();
    let mut part2 = 0;
    for k in keys {
        let parts = &gears[k];
        if parts.len() == 2 {
            println!("gear at {:?}: {:?}", k, parts);
            part2 += parts[0] * parts[1];
        }
    }
    println!("part 2: {part2}");
}
