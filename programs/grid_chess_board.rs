// Chess board printing with an 8x8 array of Option<Piece>, plus a byte board.
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Kind {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Color {
    White,
    Black,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Piece {
    kind: Kind,
    color: Color,
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let c = match self.kind {
            Kind::King => 'k',
            Kind::Queen => 'q',
            Kind::Rook => 'r',
            Kind::Bishop => 'b',
            Kind::Knight => 'n',
            Kind::Pawn => 'p',
        };
        let c = if self.color == Color::White { c.to_ascii_uppercase() } else { c };
        write!(f, "{}", c)
    }
}

struct Board([[Option<Piece>; 8]; 8]);

impl Board {
    fn initial() -> Board {
        let back = [
            Kind::Rook,
            Kind::Knight,
            Kind::Bishop,
            Kind::Queen,
            Kind::King,
            Kind::Bishop,
            Kind::Knight,
            Kind::Rook,
        ];
        let mut squares = [[None; 8]; 8];
        for (file, kind) in back.iter().enumerate() {
            squares[0][file] = Some(Piece { kind: *kind, color: Color::Black });
            squares[1][file] = Some(Piece { kind: Kind::Pawn, color: Color::Black });
            squares[6][file] = Some(Piece { kind: Kind::Pawn, color: Color::White });
            squares[7][file] = Some(Piece { kind: *kind, color: Color::White });
        }
        Board(squares)
    }

    fn mv(&mut self, from: &str, to: &str) {
        let (fr, fc) = parse_square(from);
        let (tr, tc) = parse_square(to);
        let piece = self.0[fr][fc].take();
        self.0[tr][tc] = piece;
    }

    fn count(&self, color: Color) -> usize {
        self.0
            .iter()
            .flatten()
            .filter(|p| matches!(p, Some(pc) if pc.color == color))
            .count()
    }
}

fn parse_square(s: &str) -> (usize, usize) {
    let b = s.as_bytes();
    let file = (b[0] - b'a') as usize;
    let rank = (b[1] - b'1') as usize;
    (7 - rank, file)
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (r, row) in self.0.iter().enumerate() {
            write!(f, "{} ", 8 - r)?;
            for (c, sq) in row.iter().enumerate() {
                match sq {
                    Some(p) => write!(f, "{}", p)?,
                    None => write!(f, "{}", if (r + c) % 2 == 0 { '.' } else { ':' })?,
                }
            }
            writeln!(f)?;
        }
        write!(f, "  abcdefgh")
    }
}

fn main() {
    let mut board = Board::initial();
    println!("{}", board);
    board.mv("e2", "e4");
    board.mv("e7", "e5");
    board.mv("g1", "f3");
    board.mv("d8", "h4");
    board.mv("f3", "h4");
    println!("{}", board);
    println!("white {} black {}", board.count(Color::White), board.count(Color::Black));
    println!("{:?}", board.0[4][7]);
    println!("{:?}", board.0[3][4]);

    let mut shade = [[0u8; 8]; 8];
    for r in 0..8 {
        for c in 0..8 {
            shade[r][c] = ((r + c) % 2) as u8;
        }
    }
    let dark: u32 = shade.iter().map(|row| row.iter().map(|&x| x as u32).sum::<u32>()).sum();
    println!("dark squares: {}", dark);
    println!("{:?}", shade[0]);
}
