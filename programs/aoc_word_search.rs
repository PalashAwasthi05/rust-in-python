// Day 4 2024: XMAS word search with direction vectors and get().and_then().
const INPUT: &str = r"MMMSXXMASM
MSAMXMSMSA
AMXSXMAAMM
MSAMASMSMX
XMASAMXAMM
XXAMMXXAMA
SMSMSASXSS
SAXAMASAAA
MAMMMXMMMM
MXMXAXMASX";

const DIRS: [(i32, i32); 8] = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)];

fn at(grid: &[&[u8]], r: i32, c: i32) -> Option<u8> {
    if r < 0 || c < 0 {
        return None;
    }
    grid.get(r as usize).and_then(|row| row.get(c as usize)).copied()
}

fn main() {
    let grid: Vec<&[u8]> = INPUT.lines().map(str::as_bytes).collect();
    let word = b"XMAS";
    let mut part1 = 0;
    for r in 0..grid.len() as i32 {
        for c in 0..grid[0].len() as i32 {
            for &(dr, dc) in DIRS.iter() {
                if (0..4).all(|k| at(&grid, r + dr * k, c + dc * k) == Some(word[k as usize])) {
                    part1 += 1;
                }
            }
        }
    }
    println!("part 1: {part1}");
    let mut part2 = 0;
    for r in 1..grid.len() - 1 {
        for c in 1..grid[0].len() - 1 {
            if grid[r][c] != b'A' {
                continue;
            }
            let d1 = [grid[r - 1][c - 1], grid[r + 1][c + 1]];
            let d2 = [grid[r - 1][c + 1], grid[r + 1][c - 1]];
            let ok = |d: [u8; 2]| d == [b'M', b'S'] || d == [b'S', b'M'];
            if ok(d1) && ok(d2) {
                part2 += 1;
            }
        }
    }
    println!("part 2: {part2}");
    let x_count = grid.iter().flat_map(|row| row.iter()).filter(|&&b| b == b'X').count();
    let upper = grid[0].iter().map(|b| b.to_ascii_lowercase() as char).collect::<String>();
    println!("X count {x_count}, first row lower {upper}, alpha {}", grid[1].iter().all(u8::is_ascii_alphabetic));
}
