// Day 17 2023: Dijkstra with a custom Ord state (min-heap by cost) and movement constraints.
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

const INPUT: &str = r"2413432311323
3215453535623
3255245654254
3446585845452
4546657867536
1438598798454
4457876987766
3637877979653
4654967986887
4564679986453
1224686865563
2546548887735
4322674655533";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Heading {
    Horizontal,
    Vertical,
}

#[derive(Debug, PartialEq, Eq)]
struct State {
    cost: u32,
    pos: (usize, usize),
    heading: Heading,
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost).then_with(|| self.pos.cmp(&other.pos))
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn solve(grid: &[Vec<u32>], min_run: usize, max_run: usize) -> Option<u32> {
    let (h, w) = (grid.len(), grid[0].len());
    let mut best: HashMap<((usize, usize), Heading), u32> = HashMap::new();
    let mut heap = BinaryHeap::new();
    for heading in [Heading::Horizontal, Heading::Vertical] {
        heap.push(State { cost: 0, pos: (0, 0), heading });
    }
    while let Some(State { cost, pos, heading }) = heap.pop() {
        if pos == (h - 1, w - 1) {
            return Some(cost);
        }
        if best.get(&(pos, heading)).is_some_and(|&b| b < cost) {
            continue;
        }
        let turn = match heading {
            Heading::Horizontal => Heading::Vertical,
            Heading::Horizontal | Heading::Vertical => Heading::Horizontal,
        };
        let deltas: [(isize, isize); 2] = match turn {
            Heading::Vertical => [(1, 0), (-1, 0)],
            Heading::Horizontal => [(0, 1), (0, -1)],
        };
        for (dr, dc) in deltas {
            let mut extra = 0;
            for step in 1..=max_run as isize {
                let r = pos.0 as isize + dr * step;
                let c = pos.1 as isize + dc * step;
                if r < 0 || c < 0 || r >= h as isize || c >= w as isize {
                    break;
                }
                let next = (r as usize, c as usize);
                extra += grid[next.0][next.1];
                if (step as usize) < min_run {
                    continue;
                }
                let nc = cost + extra;
                let key = (next, turn);
                if best.get(&key).map_or(true, |&b| nc < b) {
                    best.insert(key, nc);
                    heap.push(State { cost: nc, pos: next, heading: turn });
                }
            }
        }
    }
    None
}

fn main() {
    let grid: Vec<Vec<u32>> = INPUT.lines().map(|l| l.chars().map(|c| c.to_digit(10).unwrap()).collect()).collect();
    println!("part 1: {:?}", solve(&grid, 1, 3));
    println!("part 2: {:?}", solve(&grid, 4, 10));
    let mut heap: BinaryHeap<State> = vec![
        State { cost: 5, pos: (0, 1), heading: Heading::Vertical },
        State { cost: 2, pos: (3, 3), heading: Heading::Horizontal },
        State { cost: 2, pos: (1, 0), heading: Heading::Vertical },
    ]
    .into();
    while let Some(s) = heap.pop() {
        println!("{:?}", s);
    }
}
