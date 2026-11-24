// Day 10 2023: follow a pipe loop, then shoelace + Pick's theorem for the enclosed area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Dir {
    N,
    E,
    S,
    W,
}

impl Dir {
    fn delta(self) -> (i64, i64) {
        match self {
            Dir::N => (-1, 0),
            Dir::E => (0, 1),
            Dir::S => (1, 0),
            Dir::W => (0, -1),
        }
    }
    fn opposite(self) -> Dir {
        match self {
            Dir::N => Dir::S,
            Dir::S => Dir::N,
            Dir::E => Dir::W,
            Dir::W => Dir::E,
        }
    }
}

fn connections(c: char) -> Option<[Dir; 2]> {
    Some(match c {
        '|' => [Dir::N, Dir::S],
        '-' => [Dir::E, Dir::W],
        'L' => [Dir::N, Dir::E],
        'J' => [Dir::N, Dir::W],
        '7' => [Dir::S, Dir::W],
        'F' => [Dir::S, Dir::E],
        _ => return None,
    })
}

const INPUT: &str = r"FF7FSF7F7F7F7F7F---7
L|LJ||||||||||||F--J
FL-7LJLJ||||||LJL-77
F--JF--7||LJLJ7F7FJ-
L---JF-JLJ.||-FJLJJ7
|F|F-JF---7F7-L7L|7|
|FFJF7L7F-JF7|JL---7
7-L-JL7||F7|L7F-7F7|
L.L7LFJ|||||FJL7||LJ
L7JLJL-JLJLJL--JLJ.L";

fn main() {
    let grid: Vec<Vec<char>> = INPUT.lines().map(|l| l.chars().collect()).collect();
    let (sr, sc) = grid
        .iter()
        .enumerate()
        .find_map(|(r, row)| row.iter().position(|&c| c == 'S').map(|c| (r as i64, c as i64)))
        .unwrap();
    let get = |r: i64, c: i64| -> char {
        if r < 0 || c < 0 {
            return '.';
        }
        *grid.get(r as usize).and_then(|row| row.get(c as usize)).unwrap_or(&'.')
    };
    let start_dir = [Dir::N, Dir::E, Dir::S, Dir::W]
        .into_iter()
        .find(|d| {
            let (dr, dc) = d.delta();
            connections(get(sr + dr, sc + dc)).map_or(false, |cs| cs.contains(&d.opposite()))
        })
        .expect("no exit from S");
    let mut vertices: Vec<(i64, i64)> = vec![(sr, sc)];
    let (mut r, mut c, mut dir) = (sr, sc, start_dir);
    let mut steps = 0i64;
    loop {
        let (dr, dc) = dir.delta();
        r += dr;
        c += dc;
        steps += 1;
        let tile = get(r, c);
        if tile == 'S' {
            break;
        }
        let [a, b] = connections(tile).unwrap();
        let came_from = dir.opposite();
        dir = if a == came_from { b } else { a };
        if matches!(tile, 'L' | 'J' | '7' | 'F') {
            vertices.push((r, c));
        }
    }
    println!("loop length {steps}, {} corners, start {:?} heading {:?}", vertices.len(), (sr, sc), start_dir);
    println!("part 1: {}", steps / 2);
    let n = vertices.len();
    let twice_area: i64 = (0..n)
        .map(|i| {
            let (y1, x1) = vertices[i];
            let (y2, x2) = vertices[(i + 1) % n];
            x1 * y2 - x2 * y1
        })
        .sum::<i64>()
        .abs();
    let interior = twice_area / 2 - steps / 2 + 1;
    println!("part 2: {interior}");
}
