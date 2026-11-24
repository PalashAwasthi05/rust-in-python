// Day 12 2022: BFS on a height grid.
use std::collections::{HashSet, VecDeque};

const INPUT: &str = r"Sabqponm
abcryxxl
accszExk
acctuvwj
abdefghi";

fn find(grid: &[Vec<char>], target: char) -> (usize, usize) {
    for (r, row) in grid.iter().enumerate() {
        if let Some(c) = row.iter().position(|&ch| ch == target) {
            return (r, c);
        }
    }
    unreachable!()
}

fn height(c: char) -> u8 {
    match c {
        'S' => b'a',
        'E' => b'z',
        _ => c as u8,
    }
}

fn bfs(grid: &[Vec<char>], end: (usize, usize), is_start: impl Fn(char) -> bool) -> Option<usize> {
    // Walk backwards from the end so one search answers both parts.
    let rows = grid.len();
    let cols = grid[0].len();
    let mut seen = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back((end, 0));
    seen.insert(end);
    while let Some(((r, c), d)) = queue.pop_front() {
        if is_start(grid[r][c]) {
            return Some(d);
        }
        let here = height(grid[r][c]);
        let dirs: [(isize, isize); 4] = [(0, 1), (1, 0), (0, -1), (-1, 0)];
        for (dr, dc) in dirs {
            let nr = r as isize + dr;
            let nc = c as isize + dc;
            if nr < 0 || nc < 0 || nr >= rows as isize || nc >= cols as isize {
                continue;
            }
            let next = (nr as usize, nc as usize);
            if height(grid[next.0][next.1]) + 1 >= here && seen.insert(next) {
                queue.push_back((next, d + 1));
            }
        }
    }
    None
}

fn main() {
    let grid: Vec<Vec<char>> = INPUT.lines().map(|l| l.chars().collect()).collect();
    let start = find(&grid, 'S');
    let end = find(&grid, 'E');
    println!("start {:?}, end {:?}", start, end);
    println!("part 1: {}", bfs(&grid, end, |c| c == 'S').unwrap());
    println!("part 2: {}", bfs(&grid, end, |c| height(c) == b'a').unwrap());
}
