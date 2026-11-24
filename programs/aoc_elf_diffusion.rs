// Day 23 2022: elves spreading out with proposals, rotating direction order.
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

const INPUT: &str = r"....#..
..###.#
#...##.
.#...##
#.###..
##.#.##
.#..#..";

type Pos = (i32, i32);

fn bounds(elves: &HashSet<Pos>) -> (i32, i32, i32, i32) {
    let min_r = elves.iter().map(|p| p.0).min().unwrap();
    let max_r = elves.iter().map(|p| p.0).max().unwrap();
    let min_c = elves.iter().map(|p| p.1).min().unwrap();
    let max_c = elves.iter().map(|p| p.1).max().unwrap();
    (min_r, max_r, min_c, max_c)
}

fn round(elves: &mut HashSet<Pos>, order: &VecDeque<usize>) -> bool {
    // N, S, W, E: three cells to check and the step to take.
    let checks: [([Pos; 3], Pos); 4] = [
        ([(-1, -1), (-1, 0), (-1, 1)], (-1, 0)),
        ([(1, -1), (1, 0), (1, 1)], (1, 0)),
        ([(-1, -1), (0, -1), (1, -1)], (0, -1)),
        ([(-1, 1), (0, 1), (1, 1)], (0, 1)),
    ];
    let mut proposals: HashMap<Pos, Vec<Pos>> = HashMap::new();
    for &(r, c) in elves.iter() {
        let free = |dr: i32, dc: i32| !elves.contains(&(r + dr, c + dc));
        let alone = (-1..=1).all(|dr| (-1..=1).all(|dc| (dr == 0 && dc == 0) || free(dr, dc)));
        if alone {
            continue;
        }
        for &i in order {
            let (cells, (sr, sc)) = checks[i];
            if cells.iter().all(|&(dr, dc)| free(dr, dc)) {
                proposals.entry((r + sr, c + sc)).or_insert_with(Vec::new).push((r, c));
                break;
            }
        }
    }
    let mut moved = false;
    for (target, from) in proposals {
        if let [single] = from[..] {
            elves.remove(&single);
            elves.insert(target);
            moved = true;
        }
    }
    moved
}

fn main() {
    let mut elves: HashSet<Pos> = HashSet::new();
    for (r, line) in INPUT.lines().enumerate() {
        for (c, ch) in line.chars().enumerate() {
            if ch == '#' {
                elves.insert((r as i32, c as i32));
            }
        }
    }
    let mut order: VecDeque<usize> = (0..4).collect();
    let mut n = 0;
    loop {
        n += 1;
        let moved = round(&mut elves, &order);
        order.rotate_left(1);
        if n == 10 {
            let (r0, r1, c0, c1) = bounds(&elves);
            let empty = (r1 - r0 + 1) * (c1 - c0 + 1) - elves.len() as i32;
            println!("part 1: {empty}");
            for r in r0..=r1 {
                println!("{}", (c0..=c1).map(|c| if elves.contains(&(r, c)) { '#' } else { '.' }).collect::<String>());
            }
        }
        if !moved {
            break;
        }
    }
    println!("part 2: {n}");
    let mut per_row: BTreeMap<i32, usize> = BTreeMap::new();
    for &(r, _) in &elves {
        *per_row.entry(r).or_insert(0) += 1;
    }
    println!("{:?}", per_row);
    let busiest = per_row.iter().max_by_key(|&(r, n)| (n, -r)).unwrap();
    println!("busiest row {:?}, rows below 3: {:?}", busiest, per_row.range(..3).map(|(k, _)| *k).collect::<Vec<_>>());
}
