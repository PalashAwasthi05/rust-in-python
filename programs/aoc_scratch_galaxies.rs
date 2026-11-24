// Day 4 2023 (scratchcard copies) + Day 11 2023 (expanding universe distances).
use std::collections::HashSet;

const CARDS: &str = r"Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53
Card 2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19
Card 3:  1 21 53 59 44 | 69 82 63 72 16 21 14  1
Card 4: 41 92 73 84 69 | 59 84 76 51 58  5 54 83
Card 5: 87 83 26 28 32 | 88 30 70 12 93 22 82 36
Card 6: 31 18 13 56 72 | 74 77 10 23 35 67 36 11";

const SKY: &str = r"...#......
.......#..
#.........
..........
......#...
.#........
.........#
..........
.......#..
#...#.....";

fn distances(galaxies: &[(usize, usize)], empty_rows: &[usize], empty_cols: &[usize], factor: u64) -> u64 {
    let mut total = 0;
    for (i, &(r1, c1)) in galaxies.iter().enumerate() {
        for &(r2, c2) in &galaxies[i + 1..] {
            let (rlo, rhi) = (r1.min(r2), r1.max(r2));
            let (clo, chi) = (c1.min(c2), c1.max(c2));
            let er = empty_rows.iter().filter(|&&r| rlo < r && r < rhi).count() as u64;
            let ec = empty_cols.iter().filter(|&&c| clo < c && c < chi).count() as u64;
            total += (rhi - rlo + chi - clo) as u64 + (er + ec) * (factor - 1);
        }
    }
    total
}

fn main() {
    let matches: Vec<usize> = CARDS
        .lines()
        .map(|line| {
            let (_, numbers) = line.split_once(": ").unwrap();
            let (win, have) = numbers.split_once(" | ").unwrap();
            let win: HashSet<u32> = win.split_whitespace().map(|n| n.parse().unwrap()).collect();
            let have: HashSet<u32> = have.split_whitespace().map(|n| n.parse().unwrap()).collect();
            win.intersection(&have).count()
        })
        .collect();
    let points: u32 = matches.iter().filter(|&&m| m > 0).map(|&m| 1 << (m - 1)).sum();
    println!("matches {:?}", matches);
    println!("points: {points}");
    let mut copies = vec![1u32; matches.len()];
    for i in 0..matches.len() {
        for j in i + 1..=(i + matches[i]).min(matches.len() - 1) {
            copies[j] += copies[i];
        }
    }
    println!("cards: {} {:?}", copies.iter().sum::<u32>(), copies);

    let grid: Vec<&[u8]> = SKY.lines().map(|l| l.as_bytes()).collect();
    let galaxies: Vec<(usize, usize)> = grid
        .iter()
        .enumerate()
        .flat_map(|(r, row)| row.iter().enumerate().filter(|(_, &b)| b == b'#').map(move |(c, _)| (r, c)))
        .collect();
    let empty_rows: Vec<usize> = (0..grid.len()).filter(|&r| !grid[r].contains(&b'#')).collect();
    let empty_cols: Vec<usize> = (0..grid[0].len()).filter(|&c| grid.iter().all(|row| row[c] == b'.')).collect();
    println!("{} galaxies, empty rows {:?}, empty cols {:?}", galaxies.len(), empty_rows, empty_cols);
    for factor in [2, 10, 100, 1_000_000] {
        println!("factor {factor:>7}: {}", distances(&galaxies, &empty_rows, &empty_cols, factor));
    }
}
