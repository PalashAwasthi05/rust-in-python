// Sudoku solver with bitmask candidates, naked-single propagation, and
// most-constrained-cell backtracking; plus an N-queens counter for comparison.
use std::fmt;
use std::str::FromStr;

#[derive(Clone)]
struct Board {
    cells: [u8; 81],
    rows: [u16; 9],
    cols: [u16; 9],
    boxes: [u16; 9],
}

#[derive(Debug, PartialEq)]
enum BoardError {
    WrongLength(usize),
    BadChar(char),
    Conflict { row: usize, col: usize, digit: u8 },
}

fn box_of(r: usize, c: usize) -> usize {
    (r / 3) * 3 + c / 3
}

impl FromStr for Board {
    type Err = BoardError;

    fn from_str(s: &str) -> Result<Board, BoardError> {
        let chars: Vec<char> = s.chars().filter(|c| !c.is_whitespace()).collect();
        if chars.len() != 81 {
            return Err(BoardError::WrongLength(chars.len()));
        }
        let mut b = Board { cells: [0; 81], rows: [0; 9], cols: [0; 9], boxes: [0; 9] };
        for (i, &ch) in chars.iter().enumerate() {
            let d = match ch {
                '.' | '0' => continue,
                '1'..='9' => ch as u8 - b'0',
                other => return Err(BoardError::BadChar(other)),
            };
            let (r, c) = (i / 9, i % 9);
            if !b.allowed(r, c, d) {
                return Err(BoardError::Conflict { row: r + 1, col: c + 1, digit: d });
            }
            b.place(i, d);
        }
        Ok(b)
    }
}

impl Board {
    fn allowed(&self, r: usize, c: usize, d: u8) -> bool {
        let bit = 1u16 << d;
        (self.rows[r] | self.cols[c] | self.boxes[box_of(r, c)]) & bit == 0
    }

    fn place(&mut self, i: usize, d: u8) {
        let (r, c) = (i / 9, i % 9);
        let bit = 1 << d;
        self.cells[i] = d;
        self.rows[r] |= bit;
        self.cols[c] |= bit;
        self.boxes[box_of(r, c)] |= bit;
    }

    fn unplace(&mut self, i: usize) {
        let (r, c) = (i / 9, i % 9);
        let mask = !(1u16 << self.cells[i]);
        self.cells[i] = 0;
        self.rows[r] &= mask;
        self.cols[c] &= mask;
        self.boxes[box_of(r, c)] &= mask;
    }

    fn candidates(&self, i: usize) -> u16 {
        let (r, c) = (i / 9, i % 9);
        !(self.rows[r] | self.cols[c] | self.boxes[box_of(r, c)]) & 0b11_1111_1110
    }

    /// Fill every cell that has exactly one candidate; returns the placements made, or None on contradiction.
    fn propagate(&mut self) -> Option<Vec<usize>> {
        let mut placed = Vec::new();
        loop {
            let mut progress = false;
            for i in 0..81 {
                if self.cells[i] != 0 {
                    continue;
                }
                let cand = self.candidates(i);
                match cand.count_ones() {
                    0 => {
                        for &j in placed.iter().rev() {
                            self.unplace(j);
                        }
                        return None;
                    }
                    1 => {
                        self.place(i, cand.trailing_zeros() as u8);
                        placed.push(i);
                        progress = true;
                    }
                    _ => {}
                }
            }
            if !progress {
                return Some(placed);
            }
        }
    }

    fn solve(&mut self, stats: &mut SolveStats) -> bool {
        stats.nodes += 1;
        let placed = match self.propagate() {
            Some(p) => p,
            None => {
                stats.dead_ends += 1;
                return false;
            }
        };
        stats.propagated += placed.len();
        let best = (0..81)
            .filter(|&i| self.cells[i] == 0)
            .min_by_key(|&i| self.candidates(i).count_ones());
        let Some(i) = best else { return true };
        let cand = self.candidates(i);
        for d in 1..=9u8 {
            if cand & (1 << d) != 0 {
                self.place(i, d);
                if self.solve(stats) {
                    return true;
                }
                self.unplace(i);
            }
        }
        for &j in placed.iter().rev() {
            self.unplace(j);
        }
        false
    }

    fn is_complete(&self) -> bool {
        self.rows.iter().chain(self.cols.iter()).chain(self.boxes.iter()).all(|&m| m == 0b11_1111_1110)
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (r, row) in self.cells.chunks(9).enumerate() {
            if r % 3 == 0 && r > 0 {
                writeln!(f, "------+-------+------")?;
            }
            let groups: Vec<String> = row
                .chunks(3)
                .map(|g| g.iter().map(|&d| if d == 0 { ".".to_string() } else { d.to_string() }).collect::<Vec<_>>().join(" "))
                .collect();
            writeln!(f, "{}", groups.join(" | "))?;
        }
        Ok(())
    }
}

#[derive(Default, Debug)]
struct SolveStats {
    nodes: usize,
    propagated: usize,
    dead_ends: usize,
}

fn queens(n: usize, row: usize, cols: u32, d1: u32, d2: u32, first: &mut Option<Vec<usize>>, path: &mut Vec<usize>) -> u32 {
    if row == n {
        if first.is_none() {
            *first = Some(path.clone());
        }
        return 1;
    }
    let mut count = 0;
    let mut free = !(cols | d1 | d2) & ((1 << n) - 1);
    while free != 0 {
        let bit = free & free.wrapping_neg();
        free ^= bit;
        path.push(bit.trailing_zeros() as usize);
        count += queens(n, row + 1, cols | bit, (d1 | bit) << 1, (d2 | bit) >> 1, first, path);
        path.pop();
    }
    count
}

fn main() {
    let puzzles = [
        ("easy", "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"),
        ("medium", "..9748...7.........2.1.9.....7...24..64.1.59..98...3.....8.3.2.........6...2759.."),
        ("conflict", "55..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"),
        ("short", "123"),
        ("letters", "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..7x"),
    ];
    for (name, text) in puzzles.iter() {
        let mut board: Board = match text.parse() {
            Ok(b) => b,
            Err(e) => {
                println!("{}: cannot load: {:?}", name, e);
                continue;
            }
        };
        let givens = board.cells.iter().filter(|&&d| d != 0).count();
        let mut stats = SolveStats::default();
        let solved = board.solve(&mut stats);
        println!("{}: {} givens, solved={} complete={} {:?}", name, givens, solved, board.is_complete(), stats);
        if solved {
            print!("{}", board);
            let first_row: u32 = board.cells[..9].iter().map(|&d| d as u32).sum();
            println!("row sum {}", first_row);
        }
    }

    for n in [4usize, 5, 6, 8] {
        let mut first = None;
        let count = queens(n, 0, 0, 0, 0, &mut first, &mut Vec::new());
        let first = first.unwrap_or_default();
        let picture: Vec<String> = first
            .iter()
            .map(|&c| (0..n).map(|k| if k == c { 'Q' } else { '.' }).collect())
            .collect();
        println!("{}-queens: {} solutions; first {:?} {}", n, count, first, picture.join("/"));
    }
}
