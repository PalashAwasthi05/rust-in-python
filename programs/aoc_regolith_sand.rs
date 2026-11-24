// Day 14 2022: falling sand with a HashSet of blocked cells and labelled loops.
use std::collections::HashSet;

const INPUT: &str = r"498,4 -> 498,6 -> 496,6
503,4 -> 502,4 -> 502,9 -> 494,9";

fn parse(input: &str) -> HashSet<(i32, i32)> {
    let mut rock = HashSet::new();
    for line in input.lines() {
        let points: Vec<(i32, i32)> = line
            .split(" -> ")
            .map(|p| {
                let (x, y) = p.split_once(',').unwrap();
                (x.parse().unwrap(), y.parse().unwrap())
            })
            .collect();
        for pair in points.windows(2) {
            let ((x1, y1), (x2, y2)) = (pair[0], pair[1]);
            for x in x1.min(x2)..=x1.max(x2) {
                for y in y1.min(y2)..=y1.max(y2) {
                    rock.insert((x, y));
                }
            }
        }
    }
    rock
}

fn pour(rock: &HashSet<(i32, i32)>, floor: bool) -> (usize, HashSet<(i32, i32)>) {
    let max_y = rock.iter().map(|&(_, y)| y).max().unwrap();
    let mut blocked = rock.clone();
    let mut sand = HashSet::new();
    'grain: loop {
        let (mut x, mut y) = (500, 0);
        if blocked.contains(&(x, y)) {
            break;
        }
        loop {
            if y > max_y {
                if !floor {
                    break 'grain;
                }
                break;
            }
            if let Some(dx) = [0, -1, 1].into_iter().find(|dx| !blocked.contains(&(x + dx, y + 1))) {
                x += dx;
                y += 1;
            } else {
                break;
            }
        }
        blocked.insert((x, y));
        sand.insert((x, y));
    }
    (sand.len(), sand)
}

fn main() {
    let rock = parse(INPUT);
    let (p1, sand) = pour(&rock, false);
    println!("part 1: {p1}");
    let (min_x, max_x) = rock.iter().chain(sand.iter()).fold((i32::MAX, i32::MIN), |(lo, hi), &(x, _)| (lo.min(x), hi.max(x)));
    for y in 0..=9 {
        let row: String = (min_x..=max_x)
            .map(|x| {
                if rock.contains(&(x, y)) {
                    '#'
                } else if sand.contains(&(x, y)) {
                    'o'
                } else {
                    '.'
                }
            })
            .collect();
        println!("{row}");
    }
    let (p2, _) = pour(&rock, true);
    println!("part 2: {p2}");
}
