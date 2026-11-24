// Day 18 2022: surface area of a 3D droplet and exterior flood fill.
use std::collections::{HashSet, VecDeque};
use std::ops::Add;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct P3 {
    x: i32,
    y: i32,
    z: i32,
}

impl Add for P3 {
    type Output = P3;
    fn add(self, o: P3) -> P3 {
        P3 { x: self.x + o.x, y: self.y + o.y, z: self.z + o.z }
    }
}

const SIDES: [P3; 6] = [
    P3 { x: 1, y: 0, z: 0 },
    P3 { x: -1, y: 0, z: 0 },
    P3 { x: 0, y: 1, z: 0 },
    P3 { x: 0, y: -1, z: 0 },
    P3 { x: 0, y: 0, z: 1 },
    P3 { x: 0, y: 0, z: -1 },
];

const INPUT: &str = r"2,2,2
1,2,2
3,2,2
2,1,2
2,3,2
2,2,1
2,2,3
2,2,4
2,2,6
1,2,5
3,2,5
2,1,5
2,3,5";

fn main() {
    let cubes: HashSet<P3> = INPUT
        .lines()
        .map(|l| {
            let v: Vec<i32> = l.split(',').map(|n| n.parse().unwrap()).collect();
            P3 { x: v[0], y: v[1], z: v[2] }
        })
        .collect();
    let surface = cubes
        .iter()
        .flat_map(|&c| SIDES.iter().map(move |&s| c + s))
        .filter(|n| !cubes.contains(n))
        .count();
    println!("part 1: {surface}");

    let lo = cubes.iter().fold(i32::MAX, |m, c| m.min(c.x).min(c.y).min(c.z)) - 1;
    let hi = cubes.iter().fold(i32::MIN, |m, c| m.max(c.x).max(c.y).max(c.z)) + 1;
    let inside = |p: &P3| [p.x, p.y, p.z].iter().all(|v| (lo..=hi).contains(v));
    let start = P3 { x: lo, y: lo, z: lo };
    let mut outside = HashSet::from([start]);
    let mut queue = VecDeque::from(vec![start]);
    let mut exterior = 0;
    while let Some(p) = queue.pop_front() {
        for &s in &SIDES {
            let n = p + s;
            if !inside(&n) {
                continue;
            }
            if cubes.contains(&n) {
                exterior += 1;
            } else if outside.insert(n) {
                queue.push_back(n);
            }
        }
    }
    println!("part 2: {exterior}");
    let mut sorted: Vec<&P3> = cubes.iter().collect();
    sorted.sort();
    println!("min {:?} max {:?}", sorted.first().unwrap(), sorted.last().unwrap());
    println!("bounds {lo}..={hi}, outside cells {}", outside.len());
}
