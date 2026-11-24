// Small AoC helpers: custom neighbour iterator, successors, from_fn, and basin flood fill (Day 9 2021).
use std::iter;

const INPUT: &str = r"2199943210
3987894921
9856789892
8767896789
9899965678";

struct Neighbours {
    r: usize,
    c: usize,
    rows: usize,
    cols: usize,
    k: usize,
}

impl Iterator for Neighbours {
    type Item = (usize, usize);
    fn next(&mut self) -> Option<Self::Item> {
        const D: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        while self.k < D.len() {
            let (dr, dc) = D[self.k];
            self.k += 1;
            let nr = self.r.checked_add_signed(dr);
            let nc = self.c.checked_add_signed(dc);
            if let (Some(nr), Some(nc)) = (nr, nc) {
                if nr < self.rows && nc < self.cols {
                    return Some((nr, nc));
                }
            }
        }
        None
    }
}

fn neighbours(r: usize, c: usize, grid: &[Vec<u8>]) -> Neighbours {
    Neighbours { r, c, rows: grid.len(), cols: grid[0].len(), k: 0 }
}

fn main() {
    let grid: Vec<Vec<u8>> = INPUT.lines().map(|l| l.bytes().map(|b| b - b'0').collect()).collect();
    let lows: Vec<(usize, usize)> = (0..grid.len())
        .flat_map(|r| (0..grid[0].len()).map(move |c| (r, c)))
        .filter(|&(r, c)| neighbours(r, c, &grid).all(|(nr, nc)| grid[nr][nc] > grid[r][c]))
        .collect();
    let risk: u32 = lows.iter().map(|&(r, c)| grid[r][c] as u32 + 1).sum();
    println!("low points {:?}", lows);
    println!("part 1: {risk}");

    let mut seen = vec![vec![false; grid[0].len()]; grid.len()];
    let mut sizes: Vec<usize> = lows
        .iter()
        .map(|&start| {
            let mut stack = vec![start];
            let mut size = 0;
            while let Some((r, c)) = stack.pop() {
                if seen[r][c] || grid[r][c] == 9 {
                    continue;
                }
                seen[r][c] = true;
                size += 1;
                stack.extend(neighbours(r, c, &grid));
            }
            size
        })
        .collect();
    sizes.sort_unstable_by(|a, b| b.cmp(a));
    println!("basins {:?}", sizes);
    println!("part 2: {}", sizes.iter().take(3).product::<usize>());

    let collatz: Vec<u64> = iter::successors(Some(27u64), |&n| {
        if n == 1 { None } else if n % 2 == 0 { Some(n / 2) } else { Some(3 * n + 1) }
    })
    .collect();
    println!("collatz 27: {} steps, peak {}", collatz.len() - 1, collatz.iter().max().unwrap());
    let mut state = (0u64, 1u64);
    let fib: Vec<u64> = iter::from_fn(|| {
        let out = state.0;
        state = (state.1, state.0 + state.1);
        Some(out)
    })
    .take_while(|&f| f < 1000)
    .collect();
    println!("fib {:?}", fib);
    let rep: String = iter::repeat("ab").take(3).collect::<Vec<_>>().concat();
    let digits: Vec<u32> = 9875u32.to_string().chars().filter_map(|c| c.to_digit(10)).rev().collect();
    println!("{rep} {:?} {}", digits, (1..=5).rev().skip_while(|&x| x > 3).map(|x| x.to_string()).collect::<Vec<_>>().join("-"));
}
