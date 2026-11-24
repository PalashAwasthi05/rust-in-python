// Out-of-bounds indexing into a 2D array panics with rustc's message and exit status 101.
fn cell(grid: &[[u8; 4]; 3], r: usize, c: usize) -> u8 {
    grid[r][c]
}

fn main() {
    let mut grid = [[0u8; 4]; 3];
    grid[2][3] = 9;
    println!("{}", cell(&grid, 2, 3));
    println!("{:?}", grid.get(5));
    println!("{:?}", grid[1].get(3));
    println!("{:?}", grid.get(1).and_then(|row| row.get(7)));
    let rows = grid.len();
    println!("about to read row {}", rows);
    println!("{}", cell(&grid, rows, 0));
    println!("unreachable");
}
