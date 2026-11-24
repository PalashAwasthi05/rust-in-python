// Sudoku solver: backtracking over a [[u8; 9]; 9] board passed as &mut.
type Board = [[u8; 9]; 9];

fn parse(rows: [&str; 9]) -> Board {
    let mut b = [[0u8; 9]; 9];
    for (r, line) in rows.iter().enumerate() {
        for (c, ch) in line.chars().enumerate() {
            b[r][c] = ch.to_digit(10).map_or(0, |d| d as u8);
        }
    }
    b
}

fn candidates(b: &Board, r: usize, c: usize) -> [bool; 10] {
    let mut ok = [true; 10];
    ok[0] = false;
    for i in 0..9 {
        ok[b[r][i] as usize] = false;
        ok[b[i][c] as usize] = false;
    }
    let (br, bc) = (r / 3 * 3, c / 3 * 3);
    for row in &b[br..br + 3] {
        for &v in &row[bc..bc + 3] {
            ok[v as usize] = false;
        }
    }
    ok
}

fn solve(b: &mut Board, steps: &mut u32) -> bool {
    let mut best: Option<(usize, usize, usize)> = None;
    for r in 0..9 {
        for c in 0..9 {
            if b[r][c] == 0 {
                let n = candidates(b, r, c).iter().filter(|&&x| x).count();
                if best.map_or(true, |(_, _, m)| n < m) {
                    best = Some((r, c, n));
                }
            }
        }
    }
    let (r, c, _) = match best {
        None => return true,
        Some(x) => x,
    };
    let ok = candidates(b, r, c);
    for d in 1..=9u8 {
        if ok[d as usize] {
            *steps += 1;
            b[r][c] = d;
            if solve(b, steps) {
                return true;
            }
        }
    }
    b[r][c] = 0;
    false
}

fn valid(b: &Board) -> bool {
    let full: Vec<u8> = (1..=9).collect();
    let rows_ok = b.iter().all(|row| {
        let mut v = row.to_vec();
        v.sort();
        v == full
    });
    let cols_ok = (0..9).all(|c| {
        let mut v: Vec<u8> = b.iter().map(|row| row[c]).collect();
        v.sort_unstable();
        v == full
    });
    rows_ok && cols_ok
}

fn main() {
    let mut board = parse([
        "530070000",
        "600195000",
        "098000060",
        "800060003",
        "400803001",
        "700020006",
        "060000280",
        "000419005",
        "000080079",
    ]);
    let original = board;
    let mut steps = 0;
    let solved = solve(&mut board, &mut steps);
    println!("solved: {} in {} placements", solved, steps);
    for (r, row) in board.iter().enumerate() {
        if r % 3 == 0 && r > 0 {
            println!("------+-------+------");
        }
        let groups: Vec<String> = row
            .chunks(3)
            .map(|g| g.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(" "))
            .collect();
        println!("{}", groups.join(" | "));
    }
    println!("valid: {}", valid(&board));
    let filled = original.iter().flatten().filter(|&&d| d != 0).count();
    println!("givens: {}, changed: {}", filled, board != original);
    let diag: u32 = (0..9).map(|i| board[i][i] as u32).sum();
    println!("diagonal sum: {}", diag);
}
