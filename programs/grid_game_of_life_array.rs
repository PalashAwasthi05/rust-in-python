// Conway's life on a fixed torus [[bool; W]; H] stored in a struct, with generations compared.
const W: usize = 8;
const H: usize = 6;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct World {
    cells: [[bool; W]; H],
}

impl World {
    fn from_strs(rows: [&str; H]) -> World {
        let mut cells = [[false; W]; H];
        for (r, line) in rows.iter().enumerate() {
            for (c, ch) in line.chars().enumerate() {
                cells[r][c] = ch == '#';
            }
        }
        World { cells }
    }

    fn live_neighbors(&self, r: usize, c: usize) -> usize {
        let mut n = 0;
        for dr in [H - 1, 0, 1] {
            for dc in [W - 1, 0, 1] {
                if dr == 0 && dc == 0 {
                    continue;
                }
                if self.cells[(r + dr) % H][(c + dc) % W] {
                    n += 1;
                }
            }
        }
        n
    }

    fn step(&self) -> World {
        let mut next = *self;
        for r in 0..H {
            for c in 0..W {
                let n = self.live_neighbors(r, c);
                next.cells[r][c] = matches!((self.cells[r][c], n), (true, 2) | (true, 3) | (false, 3));
            }
        }
        next
    }

    fn population(&self) -> usize {
        self.cells.iter().map(|row| row.iter().filter(|&&b| b).count()).sum()
    }
}

impl std::fmt::Display for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in &self.cells {
            let line: String = row.iter().map(|&b| if b { '#' } else { '.' }).collect();
            writeln!(f, "{}", line)?;
        }
        Ok(())
    }
}

fn main() {
    let mut w = World::from_strs([
        "........",
        "..#.....",
        "...#....",
        ".###....",
        "........",
        "........",
    ]);
    let start = w;
    let mut history = std::collections::HashSet::new();
    history.insert(w);
    for gen in 1..=24 {
        w = w.step();
        if gen % 4 == 0 {
            println!("gen {} pop {}", gen, w.population());
            print!("{}", w);
        }
        if !history.insert(w) {
            println!("repeat at gen {}", gen);
            break;
        }
    }
    println!("back to start: {}", w == start);

    let blinker = World::from_strs([
        "........",
        "........",
        "..###...",
        "........",
        "........",
        "........",
    ]);
    let b1 = blinker.step();
    print!("{}", b1);
    println!("period 2: {} changed: {}", b1.step() == blinker, b1 != blinker);
}
