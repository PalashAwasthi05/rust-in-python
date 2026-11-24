// Day 15 2021: Dijkstra over a risk grid, tiled for part 2.
use std::cmp::Reverse;
use std::collections::BinaryHeap;

const INPUT: &str = r"1163751742
1381373672
2136511328
3694931569
7463417111
1319128137
1359912421
3125421639
1293138521
2311944581";

fn shortest(grid: &Vec<Vec<u32>>, scale: usize) -> u32 {
    let h = grid.len();
    let w = grid[0].len();
    let (rows, cols) = (h * scale, w * scale);
    let risk = |r: usize, c: usize| -> u32 {
        let base = grid[r % h][c % w] + (r / h + c / w) as u32;
        (base - 1) % 9 + 1
    };
    let mut dist = vec![u32::MAX; rows * cols];
    let mut heap = BinaryHeap::new();
    dist[0] = 0;
    heap.push(Reverse((0u32, 0usize)));
    while let Some(Reverse((d, idx))) = heap.pop() {
        if idx == rows * cols - 1 {
            return d;
        }
        if d > dist[idx] {
            continue;
        }
        let (r, c) = (idx / cols, idx % cols);
        let mut neighbours = Vec::with_capacity(4);
        if let Some(nr) = r.checked_sub(1) {
            neighbours.push((nr, c));
        }
        if let Some(nc) = c.checked_sub(1) {
            neighbours.push((r, nc));
        }
        if r + 1 < rows {
            neighbours.push((r + 1, c));
        }
        if c + 1 < cols {
            neighbours.push((r, c + 1));
        }
        for (nr, nc) in neighbours {
            let nd = d + risk(nr, nc);
            let ni = nr * cols + nc;
            if nd < dist[ni] {
                dist[ni] = nd;
                heap.push(Reverse((nd, ni)));
            }
        }
    }
    u32::MAX
}

fn main() {
    let grid: Vec<Vec<u32>> = INPUT
        .lines()
        .map(|l| l.chars().map(|c| c.to_digit(10).unwrap()).collect())
        .collect();
    println!("part 1: {}", shortest(&grid, 1));
    println!("part 2: {}", shortest(&grid, 2));
}
