// Grid puzzle helpers: parsing a map, neighbours, flood fill, transposition and rendering.
use std::collections::{HashSet, VecDeque};

const MAP: &str = "\
#########
#..#....#
#..#.##.#
#....#..#
####.#..#
#....#.E#
#S..##..#
#########";

fn parse(map: &str) -> Vec<Vec<char>> {
    map.lines().map(|l| l.chars().collect()).collect()
}

fn find(grid: &[Vec<char>], target: char) -> Option<(usize, usize)> {
    grid.iter().enumerate().find_map(|(r, row)| row.iter().position(|&c| c == target).map(|c| (r, c)))
}

fn neighbours(r: usize, c: usize, rows: usize, cols: usize) -> impl Iterator<Item = (usize, usize)> {
    let deltas: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    deltas.into_iter().filter_map(move |(dr, dc)| {
        let nr = r.checked_add_signed(dr as isize)?;
        let nc = c.checked_add_signed(dc as isize)?;
        (nr < rows && nc < cols).then_some((nr, nc))
    })
}

fn bfs(grid: &[Vec<char>], start: (usize, usize), goal: (usize, usize)) -> Option<usize> {
    let (rows, cols) = (grid.len(), grid[0].len());
    let mut seen = HashSet::from([start]);
    let mut queue = VecDeque::from([(start, 0usize)]);
    while let Some(((r, c), d)) = queue.pop_front() {
        if (r, c) == goal {
            return Some(d);
        }
        for (nr, nc) in neighbours(r, c, rows, cols) {
            if grid[nr][nc] != '#' && seen.insert((nr, nc)) {
                queue.push_back(((nr, nc), d + 1));
            }
        }
    }
    None
}

fn transpose<T: Clone>(m: &[Vec<T>]) -> Vec<Vec<T>> {
    (0..m[0].len()).map(|c| m.iter().map(|row| row[c].clone()).collect()).collect()
}

fn main() {
    let grid = parse(MAP);
    let start = find(&grid, 'S').expect("start");
    let goal = find(&grid, 'E').expect("end");
    println!("size {}x{} start {:?} goal {:?}", grid.len(), grid[0].len(), start, goal);
    println!("shortest path: {:?}", bfs(&grid, start, goal));
    let open = grid.iter().flatten().filter(|&&c| c != '#').count();
    println!("open cells: {}", open);
    let walls_per_row: Vec<usize> = grid.iter().map(|row| row.iter().filter(|&&c| c == '#').count()).collect();
    println!("walls per row: {:?}", walls_per_row);
    let t = transpose(&grid);
    for row in t.iter().take(3) {
        println!("{}", row.iter().collect::<String>());
    }
    let numbers = vec![vec![1, 2, 3], vec![4, 5, 6]];
    println!("transpose {:?}", transpose(&numbers));
    let diag: Vec<i32> = numbers.iter().enumerate().filter_map(|(i, row)| row.get(i).copied()).collect();
    println!("diagonal {:?}", diag);
    let flat: Vec<i32> = numbers.concat();
    let row_sums: Vec<i32> = numbers.iter().map(|r| r.iter().sum()).collect();
    let col_sums: Vec<i32> = (0..3).map(|c| numbers.iter().map(|r| r[c]).sum()).collect();
    println!("flat {:?} rows {:?} cols {:?}", flat, row_sums, col_sums);
    let mut canvas = vec![vec!['.'; 7]; 3];
    for (i, row) in canvas.iter_mut().enumerate() {
        row[i * 3] = '*';
        row.swap(0, 6);
    }
    let rendered: Vec<String> = canvas.iter().map(|r| r.iter().collect()).collect();
    println!("{}", rendered.join("\n"));
    let neighbours_of_corner: Vec<_> = neighbours(0, 0, 3, 3).collect();
    println!("corner neighbours {:?}", neighbours_of_corner);
    let checked = [5usize.checked_add_signed(-6), 5usize.checked_add_signed(2)];
    println!("checked_add_signed {:?}", checked);
    let spiral: Vec<(i32, i32)> = (0..8).scan((0i32, 0i32, 0usize), |state, i| {
        let dirs = [(0, 1), (1, 0), (0, -1), (-1, 0)];
        let out = (state.0, state.1);
        if i % 2 == 1 {
            state.2 = (state.2 + 1) % 4;
        }
        state.0 += dirs[state.2].0;
        state.1 += dirs[state.2].1;
        Some(out)
    }).collect();
    println!("walk {:?}", spiral);
    let mut sorted_rows = grid.clone();
    sorted_rows.sort_by_key(|r| std::cmp::Reverse(r.iter().filter(|&&c| c == '.').count()));
    println!("most open row: {}", sorted_rows[0].iter().collect::<String>());
}
