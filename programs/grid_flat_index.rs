use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Cell {
    Empty,
    Wall,
    Start,
}

struct Grid<T> {
    w: usize,
    h: usize,
    data: Vec<T>,
}

impl<T: Copy + Default> Grid<T> {
    fn new(w: usize, h: usize) -> Self {
        Grid { w, h, data: vec![T::default(); w * h] }
    }
    fn get(&self, x: usize, y: usize) -> Option<&T> {
        if x < self.w && y < self.h { self.data.get(y * self.w + x) } else { None }
    }
    fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        if x < self.w && y < self.h { self.data.get_mut(y * self.w + x) } else { None }
    }
    fn rows(&self) -> impl Iterator<Item = &[T]> + '_ {
        self.data.chunks(self.w)
    }
}

impl<T> std::ops::Index<(usize, usize)> for Grid<T> {
    type Output = T;
    fn index(&self, (x, y): (usize, usize)) -> &T {
        &self.data[y * self.w + x]
    }
}

impl<T> std::ops::IndexMut<(usize, usize)> for Grid<T> {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut T {
        &mut self.data[y * self.w + x]
    }
}

impl Default for Cell {
    fn default() -> Self {
        Cell::Empty
    }
}

fn main() {
    let mut g: Grid<i32> = Grid::new(4, 3);
    g[(1, 2)] = 5;
    *g.get_mut(3, 0).unwrap() += 7;
    if let Some(c) = g.get_mut(0, 1) {
        *c = -1;
    }
    println!("{:?} {:?}", g.get(1, 2), g.get(4, 0));
    for row in g.rows() {
        println!("{:?}", row);
    }
    g[(1, 2)] *= 3;
    println!("{}", g[(1, 2)]);

    let mut cells: Grid<Cell> = Grid::new(3, 2);
    cells[(0, 0)] = Cell::Start;
    cells[(2, 1)] = Cell::Wall;
    let mut tally: HashMap<Cell, usize> = HashMap::new();
    for c in &cells.data {
        *tally.entry(*c).or_default() += 1;
    }
    let mut t: Vec<_> = tally.into_iter().collect();
    t.sort_by_key(|(c, _)| format!("{:?}", c));
    println!("{:?}", t);

    let mut board = [[Cell::Empty; 3]; 3];
    board[1] = [Cell::Wall; 3];
    board[1][1] = Cell::Start;
    let walls = board.iter().flatten().filter(|&&c| c == Cell::Wall).count();
    println!("{} {:?}", walls, board[1]);
    let start = board
        .iter()
        .enumerate()
        .find_map(|(r, row)| row.iter().position(|&c| c == Cell::Start).map(|c| (r, c)));
    println!("{:?}", start);
    let mut dist = [[u32::MAX; 3]; 3];
    dist[1][1] = 0;
    for _ in 0..3 {
        for r in 0..3 {
            for c in 0..3 {
                let mut best = dist[r][c];
                if r > 0 { best = best.min(dist[r - 1][c].saturating_add(1)); }
                if c > 0 { best = best.min(dist[r][c - 1].saturating_add(1)); }
                if r < 2 { best = best.min(dist[r + 1][c].saturating_add(1)); }
                if c < 2 { best = best.min(dist[r][c + 1].saturating_add(1)); }
                dist[r][c] = best;
            }
        }
    }
    for row in &dist {
        println!("{:?}", row);
    }
    let mut lifes = vec![vec![0u8; 5]; 2];
    lifes[1][4] = 1;
    let copy = lifes.clone();
    lifes[0].copy_from_slice(&copy[1]);
    println!("{:?}", lifes);
    let longest = ["ab", "abcd", "abc"].iter().max_by_key(|s| s.len()).unwrap();
    println!("{}", longest);
}
