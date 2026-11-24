use std::collections::HashSet;
use std::fmt;
use std::ops::{Index, IndexMut};

const W: usize = 12;
const H: usize = 8;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum Cell {
    #[default]
    Dead,
    Alive,
}

#[derive(Clone, PartialEq, Eq)]
struct Grid {
    cells: [[Cell; W]; H],
    wrap: bool,
}

impl Index<(usize, usize)> for Grid {
    type Output = Cell;
    fn index(&self, (r, c): (usize, usize)) -> &Cell {
        &self.cells[r][c]
    }
}

impl IndexMut<(usize, usize)> for Grid {
    fn index_mut(&mut self, (r, c): (usize, usize)) -> &mut Cell {
        &mut self.cells[r][c]
    }
}

impl Grid {
    fn empty(wrap: bool) -> Self {
        Grid { cells: [[Cell::Dead; W]; H], wrap }
    }

    fn from_pattern(wrap: bool, at: (usize, usize), pattern: &[&str]) -> Self {
        let mut g = Grid::empty(wrap);
        g.stamp(at, pattern);
        g
    }

    fn stamp(&mut self, (r0, c0): (usize, usize), pattern: &[&str]) {
        for (dr, row) in pattern.iter().enumerate() {
            for (dc, ch) in row.chars().enumerate() {
                if ch == '#' || ch == 'O' {
                    self[((r0 + dr) % H, (c0 + dc) % W)] = Cell::Alive;
                }
            }
        }
    }

    fn neighbours(&self, r: usize, c: usize) -> usize {
        let mut n = 0;
        for dr in [-1i32, 0, 1] {
            for dc in [-1i32, 0, 1] {
                if dr == 0 && dc == 0 {
                    continue;
                }
                let (rr, cc) = (r as i32 + dr, c as i32 + dc);
                let pos = if self.wrap {
                    Some((rr.rem_euclid(H as i32) as usize, cc.rem_euclid(W as i32) as usize))
                } else if rr >= 0 && cc >= 0 && (rr as usize) < H && (cc as usize) < W {
                    Some((rr as usize, cc as usize))
                } else {
                    None
                };
                if let Some(p) = pos {
                    if self[p] == Cell::Alive {
                        n += 1;
                    }
                }
            }
        }
        n
    }

    fn step(&self) -> Grid {
        let mut next = Grid::empty(self.wrap);
        for r in 0..H {
            for c in 0..W {
                let n = self.neighbours(r, c);
                next.cells[r][c] = match (self.cells[r][c], n) {
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    (Cell::Dead, 3) => Cell::Alive,
                    _ => Cell::Dead,
                };
            }
        }
        next
    }

    fn population(&self) -> usize {
        self.cells.iter().flatten().filter(|&&c| c == Cell::Alive).count()
    }

    fn bounding_box(&self) -> Option<((usize, usize), (usize, usize))> {
        let live: Vec<(usize, usize)> = (0..H)
            .flat_map(|r| (0..W).map(move |c| (r, c)))
            .filter(|&p| self[p] == Cell::Alive)
            .collect();
        let rmin = live.iter().map(|p| p.0).min()?;
        let rmax = live.iter().map(|p| p.0).max()?;
        let cmin = live.iter().map(|p| p.1).min()?;
        let cmax = live.iter().map(|p| p.1).max()?;
        Some(((rmin, cmin), (rmax, cmax)))
    }

    fn signature(&self) -> u64 {
        let mut h: u64 = 1469598103934665603;
        for (i, cell) in self.cells.iter().flatten().enumerate() {
            if *cell == Cell::Alive {
                h ^= i as u64;
                h = h.wrapping_mul(1099511628211);
            }
        }
        h
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "+{}+", "-".repeat(W))?;
        for row in &self.cells {
            let line: String = row.iter().map(|c| if *c == Cell::Alive { '#' } else { '.' }).collect();
            writeln!(f, "|{}|", line)?;
        }
        write!(f, "+{}+", "-".repeat(W))
    }
}

struct Simulation {
    name: &'static str,
    grid: Grid,
    history: Vec<u64>,
}

#[derive(Debug)]
enum Outcome {
    Extinct(usize),
    StillLife(usize),
    Oscillator { period: usize, from: usize },
    Running(usize),
}

impl Simulation {
    fn run(&mut self, max_gen: usize, show_at: &[usize]) -> Outcome {
        let mut seen = HashSet::new();
        for gen in 0..=max_gen {
            if show_at.contains(&gen) {
                println!("{} gen {} pop {}:\n{}", self.name, gen, self.grid.population(), self.grid);
            }
            if self.grid.population() == 0 {
                return Outcome::Extinct(gen);
            }
            let sig = self.grid.signature();
            if !seen.insert(sig) {
                let first = self.history.iter().position(|&s| s == sig).unwrap();
                let period = gen - first;
                return if period == 1 { Outcome::StillLife(first) } else { Outcome::Oscillator { period, from: first } };
            }
            self.history.push(sig);
            self.grid = self.grid.step();
        }
        Outcome::Running(max_gen)
    }
}

fn main() {
    let glider = [".#.", "..#", "###"];
    let blinker = ["###"];
    let toad = [".###", "###."];
    let block = ["##", "##"];
    let r_pentomino = [".##", "##.", ".#."];
    let diehard = ["......#.", "##......", ".#...###"];
    let pulsar_quarter = ["..###", "", "#....#", "#....#", "#....#", "..###"];

    let mut sims = vec![
        Simulation { name: "glider", grid: Grid::from_pattern(true, (0, 0), &glider), history: vec![] },
        Simulation { name: "blinker", grid: Grid::from_pattern(false, (3, 5), &blinker), history: vec![] },
        Simulation { name: "toad", grid: Grid::from_pattern(false, (3, 6), &toad), history: vec![] },
        Simulation { name: "block", grid: Grid::from_pattern(false, (0, 0), &block), history: vec![] },
        Simulation { name: "r-pentomino", grid: Grid::from_pattern(false, (2, 6), &r_pentomino), history: vec![] },
        Simulation { name: "diehard", grid: Grid::from_pattern(false, (2, 4), &diehard), history: vec![] },
        Simulation { name: "odd shape", grid: Grid::from_pattern(true, (1, 5), &pulsar_quarter), history: vec![] },
    ];

    for sim in sims.iter_mut() {
        let show: &[usize] = match sim.name {
            "glider" => &[0, 4],
            "r-pentomino" => &[0, 7],
            _ => &[],
        };
        let outcome = sim.run(7, show);
        let bbox = sim.grid.bounding_box();
        println!("{:<12} -> {:?}, final pop {}, bbox {:?}", sim.name, outcome, sim.grid.population(), bbox);
    }

    // A glider moves one cell diagonally every four generations, wrapping on the torus.
    let mut g = Grid::from_pattern(true, (H - 1, W - 1), &glider);
    let start = g.clone();
    for gen in 1..=4 {
        g = g.step();
        println!("gen {} pop {} same-as-start {}", gen, g.population(), g == start);
    }
    let expected = Grid::from_pattern(true, (0, 0), &glider);
    println!("glider wrapped to the origin: {}", g == expected);
    println!("default cell is {:?}", Cell::default());
}
