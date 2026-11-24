// Day 9 2022: rope simulation with a HashSet of visited positions.
use std::collections::HashSet;

const INPUT: &str = r"R 5
U 8
L 8
D 3
R 17
D 10
L 25
U 20";

fn simulate(moves: &[(char, i32)], knots: usize) -> usize {
    let mut rope = vec![(0i32, 0i32); knots];
    let mut visited: HashSet<(i32, i32)> = HashSet::new();
    visited.insert((0, 0));
    for &(dir, steps) in moves {
        let (dx, dy) = match dir {
            'R' => (1, 0),
            'L' => (-1, 0),
            'U' => (0, 1),
            'D' => (0, -1),
            _ => unreachable!(),
        };
        for _ in 0..steps {
            rope[0].0 += dx;
            rope[0].1 += dy;
            for i in 1..knots {
                let (hx, hy) = rope[i - 1];
                let (tx, ty) = &mut rope[i];
                let (ddx, ddy) = (hx - *tx, hy - *ty);
                if ddx.abs() > 1 || ddy.abs() > 1 {
                    *tx += ddx.signum();
                    *ty += ddy.signum();
                }
            }
            visited.insert(*rope.last().unwrap());
        }
    }
    visited.len()
}

fn main() {
    let moves: Vec<(char, i32)> = INPUT
        .lines()
        .map(|l| {
            let mut parts = l.split_whitespace();
            let d = parts.next().unwrap().chars().next().unwrap();
            let n = parts.next().unwrap().parse().unwrap();
            (d, n)
        })
        .collect();
    println!("part 1: {}", simulate(&moves, 2));
    println!("part 2: {}", simulate(&moves, 10));
    let total: i32 = moves.iter().map(|&(_, n)| n).sum();
    println!("total steps: {total}");
}
