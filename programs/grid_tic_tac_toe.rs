// Tic-tac-toe board with Option<Player> cells and win detection.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Player {
    X,
    O,
}

impl Player {
    fn other(self) -> Player {
        match self {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }
    fn symbol(self) -> char {
        match self {
            Player::X => 'X',
            Player::O => 'O',
        }
    }
}

struct Board {
    cells: [[Option<Player>; 3]; 3],
}

const LINES: [[(usize, usize); 3]; 8] = [
    [(0, 0), (0, 1), (0, 2)],
    [(1, 0), (1, 1), (1, 2)],
    [(2, 0), (2, 1), (2, 2)],
    [(0, 0), (1, 0), (2, 0)],
    [(0, 1), (1, 1), (2, 1)],
    [(0, 2), (1, 2), (2, 2)],
    [(0, 0), (1, 1), (2, 2)],
    [(0, 2), (1, 1), (2, 0)],
];

impl Board {
    fn new() -> Self {
        Board { cells: [[None; 3]; 3] }
    }

    fn play(&mut self, r: usize, c: usize, p: Player) -> Result<(), String> {
        match self.cells[r][c] {
            Some(q) => Err(format!("cell ({}, {}) taken by {:?}", r, c, q)),
            None => {
                self.cells[r][c] = Some(p);
                Ok(())
            }
        }
    }

    fn winner(&self) -> Option<Player> {
        for line in LINES.iter() {
            let [a, b, c] = *line;
            if let Some(p) = self.cells[a.0][a.1] {
                if self.cells[b.0][b.1] == Some(p) && self.cells[c.0][c.1] == Some(p) {
                    return Some(p);
                }
            }
        }
        None
    }

    fn full(&self) -> bool {
        self.cells.iter().all(|row| row.iter().all(|c| c.is_some()))
    }

    fn show(&self) {
        for (i, row) in self.cells.iter().enumerate() {
            let s: Vec<String> = row
                .iter()
                .map(|c| c.map_or('.', |p| p.symbol()).to_string())
                .collect();
            println!(" {} ", s.join(" | "));
            if i < 2 {
                println!("---+---+---");
            }
        }
    }
}

fn run_game(moves: &[(usize, usize)]) {
    let mut board = Board::new();
    let mut turn = Player::X;
    for &(r, c) in moves {
        if let Err(e) = board.play(r, c, turn) {
            println!("error: {}", e);
            continue;
        }
        if let Some(w) = board.winner() {
            board.show();
            println!("{:?} wins!", w);
            return;
        }
        turn = turn.other();
    }
    board.show();
    if board.full() {
        println!("draw");
    } else {
        println!("unfinished, {:?} to move", turn);
    }
}

fn main() {
    run_game(&[(0, 0), (1, 1), (0, 1), (2, 2), (0, 2)]);
    println!();
    run_game(&[(0, 0), (1, 1), (2, 2), (0, 2), (2, 0), (1, 0), (1, 2), (2, 1), (0, 1)]);
    println!();
    run_game(&[(1, 1), (1, 1), (0, 0), (2, 0), (0, 2), (1, 0)]);
}
