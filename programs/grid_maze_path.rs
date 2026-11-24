// Maze shortest path with BFS over a byte grid, distance grid, and path reconstruction.
use std::collections::VecDeque;

const MAZE: [&str; 7] = [
    "#########",
    "#S..#...#",
    "#.#.#.#.#",
    "#.#...#.#",
    "#.#####.#",
    "#......E#",
    "#########",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tile {
    Wall,
    Open,
}

fn main() {
    let h = MAZE.len();
    let w = MAZE[0].len();
    let mut tiles = vec![vec![Tile::Wall; w]; h];
    let mut start = (0, 0);
    let mut end = (0, 0);
    for (r, line) in MAZE.iter().enumerate() {
        for (c, b) in line.bytes().enumerate() {
            tiles[r][c] = if b == b'#' { Tile::Wall } else { Tile::Open };
            match b {
                b'S' => start = (r, c),
                b'E' => end = (r, c),
                _ => {}
            }
        }
    }
    println!("start {:?} end {:?}", start, end);

    let mut dist: Vec<Vec<Option<u32>>> = vec![vec![None; w]; h];
    let mut prev = vec![vec![None::<(usize, usize)>; w]; h];
    let mut q = VecDeque::from([start]);
    dist[start.0][start.1] = Some(0);
    while let Some((r, c)) = q.pop_front() {
        let d = dist[r][c].unwrap();
        let cand = [(r - 1, c), (r + 1, c), (r, c - 1), (r, c + 1)];
        for &(nr, nc) in cand.iter() {
            if tiles[nr][nc] == Tile::Open && dist[nr][nc].is_none() {
                dist[nr][nc] = Some(d + 1);
                prev[nr][nc] = Some((r, c));
                q.push_back((nr, nc));
            }
        }
    }
    println!("distance {:?}", dist[end.0][end.1]);

    let mut path = vec![end];
    let mut cur = end;
    while let Some(p) = prev[cur.0][cur.1] {
        path.push(p);
        cur = p;
    }
    path.reverse();
    println!("path len {}", path.len());

    let mut canvas: Vec<Vec<u8>> = MAZE.iter().map(|l| l.as_bytes().to_vec()).collect();
    for &(r, c) in &path[1..path.len() - 1] {
        canvas[r][c] = b'*';
    }
    for row in &canvas {
        println!("{}", String::from_utf8(row.clone()).unwrap());
    }
    for row in &dist {
        let line: String = row
            .iter()
            .map(|d| match d {
                Some(n) => char::from_digit(n % 10, 10).unwrap(),
                None => ' ',
            })
            .collect();
        println!("{}", line.trim_end());
    }
    let reachable = dist.iter().flatten().filter(|d| d.is_some()).count();
    let farthest = dist.iter().flatten().filter_map(|&d| d).max();
    println!("reachable {} farthest {:?}", reachable, farthest);
}
