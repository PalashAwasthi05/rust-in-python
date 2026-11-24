type Board = Vec<Vec<bool>>;

fn parse(rows: &[&str]) -> Board {
    rows.iter().map(|r| r.chars().map(|c| c == '#').collect()).collect()
}

fn neighbors(b: &Board, r: usize, c: usize) -> usize {
    let h = b.len() as i32;
    let w = b[0].len() as i32;
    let mut n = 0;
    for dr in -1..=1 {
        for dc in -1..=1 {
            if dr == 0 && dc == 0 { continue; }
            let rr = (r as i32 + dr + h) % h;
            let cc = (c as i32 + dc + w) % w;
            if b[rr as usize][cc as usize] { n += 1; }
        }
    }
    n
}

fn step(b: &Board) -> Board {
    let mut next = vec![vec![false; b[0].len()]; b.len()];
    for r in 0..b.len() {
        for c in 0..b[0].len() {
            let n = neighbors(b, r, c);
            next[r][c] = matches!((b[r][c], n), (true, 2) | (true, 3) | (false, 3));
        }
    }
    next
}

fn render(b: &Board) -> String {
    b.iter().map(|row| row.iter().map(|&x| if x { '#' } else { '.' }).collect::<String>()).collect::<Vec<_>>().join("\n")
}

fn population(b: &Board) -> usize {
    b.iter().flatten().filter(|&&x| x).count()
}

fn main() {
    let mut board = parse(&[
        "..........",
        "..#.......",
        "...#......",
        ".###......",
        "..........",
        "......###.",
        "..........",
        "..........",
    ]);
    println!("gen 0 (pop {}):\n{}", population(&board), render(&board));
    let mut history = vec![board.clone()];
    for gen in 1..=8 {
        board = step(&board);
        history.push(board.clone());
        if gen % 4 == 0 {
            println!("gen {} (pop {}):\n{}", gen, population(&board), render(&board));
        }
    }
    let pops: Vec<usize> = history.iter().map(population).collect();
    println!("populations: {:?}", pops);

    let blinker = parse(&[".....", "..#..", "..#..", "..#..", "....."]);
    let two = step(&step(&blinker));
    println!("blinker period 2: {}", two == blinker);
    let block = parse(&["....", ".##.", ".##.", "...."]);
    println!("block still life: {}", step(&block) == block);
}
