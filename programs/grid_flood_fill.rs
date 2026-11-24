// Flood fill and island counting on a 2D char grid (Vec<Vec<char>>) and on a fixed array.
use std::collections::VecDeque;

fn parse(lines: &[&str]) -> Vec<Vec<char>> {
    lines.iter().map(|l| l.chars().collect()).collect()
}

fn flood(grid: &mut Vec<Vec<char>>, r: usize, c: usize, from: char, to: char) -> usize {
    if grid[r][c] != from {
        return 0;
    }
    let mut q = VecDeque::new();
    q.push_back((r, c));
    grid[r][c] = to;
    let mut count = 0;
    let dirs: [(i32, i32); 4] = [(0, 1), (1, 0), (0, -1), (-1, 0)];
    while let Some((r, c)) = q.pop_front() {
        count += 1;
        for (dr, dc) in dirs {
            let nr = r as i32 + dr;
            let nc = c as i32 + dc;
            if nr < 0 || nc < 0 || nr as usize >= grid.len() || nc as usize >= grid[0].len() {
                continue;
            }
            let (nr, nc) = (nr as usize, nc as usize);
            if grid[nr][nc] == from {
                grid[nr][nc] = to;
                q.push_back((nr, nc));
            }
        }
    }
    count
}

fn show(grid: &[Vec<char>]) {
    for row in grid {
        println!("{}", row.iter().collect::<String>());
    }
}

const W: usize = 6;
const H: usize = 4;

fn dfs(seen: &mut [[bool; W]; H], land: &[[u8; W]; H], r: usize, c: usize) -> u32 {
    if seen[r][c] || land[r][c] == 0 {
        return 0;
    }
    seen[r][c] = true;
    let mut size = 1;
    if r > 0 {
        size += dfs(seen, land, r - 1, c);
    }
    if r + 1 < H {
        size += dfs(seen, land, r + 1, c);
    }
    if c > 0 {
        size += dfs(seen, land, r, c - 1);
    }
    if c + 1 < W {
        size += dfs(seen, land, r, c + 1);
    }
    size
}

fn main() {
    let mut grid = parse(&[
        "..##....",
        ".#..#...",
        ".#..#.##",
        "..##..#.",
        "......#.",
    ]);
    show(&grid);
    let n = flood(&mut grid, 1, 2, '.', 'o');
    println!("filled {} cells", n);
    show(&grid);
    let n = flood(&mut grid, 0, 0, '.', '~');
    println!("filled {} cells", n);
    show(&grid);
    println!("again: {}", flood(&mut grid, 0, 0, '.', '~'));

    let land: [[u8; W]; H] = [
        [1, 1, 0, 0, 0, 1],
        [1, 0, 0, 1, 0, 1],
        [0, 0, 1, 1, 0, 0],
        [1, 0, 0, 0, 0, 1],
    ];
    let mut seen = [[false; W]; H];
    let mut sizes = Vec::new();
    for r in 0..H {
        for c in 0..W {
            let s = dfs(&mut seen, &land, r, c);
            if s > 0 {
                sizes.push(s);
            }
        }
    }
    println!("islands: {} sizes {:?}", sizes.len(), sizes);
    sizes.sort_unstable_by(|a, b| b.cmp(a));
    println!("largest first: {:?}", sizes);
}
