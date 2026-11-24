// Day 4 2021: bingo with [[u32; 5]; 5] boards and a marked bitmask.
const INPUT: &str = r"7,4,9,5,11,17,23,2,0,14,21,24,10,16,13,6,15,25,12,22,18,20,8,19,3,26,1

22 13 17 11  0
 8  2 23  4 24
21  9 14 16  7
 6 10  3 18  5
 1 12 20 15 19

 3 15  0  2 22
 9 18 13 17  5
19  8  7 25 23
20 11 10 24  4
14 21 16 12  6

14 21 17 24  4
10 16 15  9 19
18  8 23 26 20
22 11 13  6  5
 2  0 12  3  7";

#[derive(Clone)]
struct Board {
    cells: [[u32; 5]; 5],
    marked: u32,
    won: bool,
}

impl Board {
    fn parse(block: &str) -> Board {
        let mut cells = [[0; 5]; 5];
        for (r, line) in block.lines().enumerate() {
            for (c, n) in line.split_whitespace().enumerate() {
                cells[r][c] = n.parse().unwrap();
            }
        }
        Board { cells, marked: 0, won: false }
    }

    fn mark(&mut self, n: u32) -> bool {
        for r in 0..5 {
            for c in 0..5 {
                if self.cells[r][c] == n {
                    self.marked |= 1 << (r * 5 + c);
                }
            }
        }
        let row = 0b11111u32;
        let col = 0b00001_00001_00001_00001_00001u32;
        (0..5).any(|i| self.marked >> (i * 5) & row == row || (self.marked >> i) & col == col)
    }

    fn unmarked_sum(&self) -> u32 {
        self.cells
            .iter()
            .flatten()
            .enumerate()
            .filter(|(i, _)| self.marked & (1 << i) == 0)
            .map(|(_, &v)| v)
            .sum()
    }
}

fn main() {
    let mut blocks = INPUT.split("\n\n");
    let draws: Vec<u32> = blocks.next().unwrap().split(',').map(|n| n.parse().unwrap()).collect();
    let mut boards: Vec<Board> = blocks.map(Board::parse).collect();
    let mut scores = Vec::new();
    for &n in &draws {
        for (i, b) in boards.iter_mut().enumerate() {
            if !b.won && b.mark(n) {
                b.won = true;
                scores.push((i, n, b.unmarked_sum() * n));
            }
        }
    }
    println!("first: {:?}", scores.first().unwrap());
    println!("last: {:?}", scores.last().unwrap());
    let b = &boards[2];
    for row in &b.cells {
        let line: Vec<String> = row.iter().map(|v| format!("{v:2}")).collect();
        println!("{}", line.join(" "));
    }
    println!("marked {:025b}", b.marked);
    let diag: u32 = (0..5).map(|i| b.cells[i][i]).sum();
    let transposed: Vec<Vec<u32>> = (0..5).map(|c| b.cells.iter().map(|row| row[c]).collect()).collect();
    println!("diag {diag}, first column {:?}", transposed[0]);
}
