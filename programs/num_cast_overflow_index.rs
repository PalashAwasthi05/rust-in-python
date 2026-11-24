// Mixing signed offsets with usize indices: the common grid-neighbour pattern.
fn neighbours(grid: &[Vec<u8>], r: usize, c: usize) -> u32 {
    let mut total = 0u32;
    for dr in -1i32..=1 {
        for dc in -1i32..=1 {
            if dr == 0 && dc == 0 { continue; }
            let nr = r as i32 + dr;
            let nc = c as i32 + dc;
            if nr < 0 || nc < 0 || nr as usize >= grid.len() || nc as usize >= grid[0].len() { continue; }
            total += grid[nr as usize][nc as usize] as u32;
        }
    }
    total
}

fn main() {
    let grid = vec![vec![1u8, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
    for r in 0..3 {
        let row: Vec<u32> = (0..3).map(|c| neighbours(&grid, r, c)).collect();
        println!("{:?}", row);
    }
    let pos: usize = 2;
    let delta: isize = -3;
    let wrapped = (pos as isize + delta).rem_euclid(grid.len() as isize) as usize;
    println!("{}", wrapped);
    println!("{:?}", pos.checked_add_signed(delta));
    println!("{}", pos.wrapping_add_signed(delta));
    println!("{}", pos.saturating_add_signed(delta));
    println!("{:?}", 5usize.checked_add_signed(-2));
    let raw = pos.wrapping_sub(3);
    println!("{}", raw);
    println!("{}", raw as isize);
    println!("{}", grid[(pos as isize + delta) as usize][0]);
}
