// Cycle detection: spin-cycle tilting of a rock grid (Day 14 2023), state -> index map.
use std::collections::HashMap;

const INPUT: &str = r"O....#....
O.OO#....#
.....##...
OO.#O....O
.O.....O#.
O.#..O.#.#
..O..#O..O
.......O..
#....###..
#OO..#....";

type Grid = Vec<Vec<u8>>;

fn tilt_north(g: &mut Grid) {
    let rows = g.len();
    for c in 0..g[0].len() {
        let mut free = 0;
        for r in 0..rows {
            match g[r][c] {
                b'#' => free = r + 1,
                b'O' => {
                    g[r][c] = b'.';
                    g[free][c] = b'O';
                    free += 1;
                }
                _ => {}
            }
        }
    }
}

fn rotate_cw(g: &Grid) -> Grid {
    let rows = g.len();
    let cols = g[0].len();
    (0..cols)
        .map(|c| (0..rows).rev().map(|r| g[r][c]).collect())
        .collect()
}

fn spin(g: &mut Grid) {
    for _ in 0..4 {
        tilt_north(g);
        *g = rotate_cw(g);
    }
}

fn load(g: &Grid) -> usize {
    let n = g.len();
    g.iter()
        .enumerate()
        .map(|(i, row)| row.iter().filter(|&&b| b == b'O').count() * (n - i))
        .sum()
}

fn main() {
    let grid: Grid = INPUT.lines().map(|l| l.bytes().collect()).collect();
    let mut p1 = grid.clone();
    tilt_north(&mut p1);
    println!("part 1: {}", load(&p1));

    let target = 1_000_000_000usize;
    let mut seen: HashMap<Grid, usize> = HashMap::new();
    let mut history: Vec<usize> = Vec::new();
    let mut g = grid;
    let mut i = 0;
    let answer = loop {
        if let Some(&first) = seen.get(&g) {
            let period = i - first;
            println!("cycle found: start {first}, period {period}");
            let idx = first + (target - first) % period;
            break history[idx];
        }
        seen.insert(g.clone(), i);
        history.push(load(&g));
        spin(&mut g);
        i += 1;
    };
    println!("part 2: {}", answer);
    println!("{}", String::from_utf8(g[0].clone()).unwrap());
}
