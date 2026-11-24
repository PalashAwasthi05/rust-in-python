// Assorted array features: Default/Hash/Ord derives with array fields, statics, each_mut, zip, comparisons with slices.
use std::collections::BTreeMap;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Pixel {
    rgba: [u8; 4],
}

#[derive(Debug, Default)]
struct Stats {
    counts: [u32; 5],
    names: [String; 2],
    grid: [[i8; 3]; 2],
}

static PRIMES: [u32; 6] = [2, 3, 5, 7, 11, 13];
const KNIGHT: [(i32, i32); 8] = [(1, 2), (2, 1), (2, -1), (1, -2), (-1, -2), (-2, -1), (-2, 1), (-1, 2)];

fn knight_moves(r: i32, c: i32) -> usize {
    KNIGHT
        .iter()
        .map(|&(dr, dc)| (r + dr, c + dc))
        .filter(|&(nr, nc)| (0..8).contains(&nr) && (0..8).contains(&nc))
        .count()
}

fn main() {
    let mut s = Stats::default();
    s.counts[2] += 3;
    s.names[0] = "left".to_string();
    s.grid[1][2] = -4;
    println!("{:?}", s);
    println!("{:#?}", s.grid);

    let mut pixels = [Pixel::default(); 3];
    pixels[1].rgba = [255, 0, 0, 255];
    pixels[2].rgba[3] = 128;
    let mut by_color: BTreeMap<Pixel, usize> = BTreeMap::new();
    for (i, p) in pixels.iter().enumerate() {
        by_color.insert(*p, i);
    }
    println!("{:?}", by_color);
    println!("{:?}", pixels.iter().max());

    println!("{} {:?}", PRIMES.len(), PRIMES.iter().rev().take(2).collect::<Vec<_>>());
    println!("{}", PRIMES.iter().sum::<u32>());

    let board: Vec<usize> = (0..8).map(|c| knight_moves(0, c)).collect();
    println!("{:?}", board);
    let mut heat = [[0usize; 8]; 8];
    for (r, row) in heat.iter_mut().enumerate() {
        for (c, cell) in row.iter_mut().enumerate() {
            *cell = knight_moves(r as i32, c as i32);
        }
    }
    for row in heat.iter().take(3) {
        println!("{}", row.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(""));
    }

    let mut a = [1, 2, 3];
    let b = [10, 20, 30];
    for (x, y) in a.iter_mut().zip(b.iter()) {
        *x += y;
    }
    println!("{:?}", a);
    for r in a.each_mut() {
        *r *= 2;
    }
    println!("{:?}", a);
    let v = vec![22, 44, 66];
    println!("{} {} {}", a == v[..], a[..] == v[..], &a[..2] == &v[..2]);
    println!("{}", a.as_slice().len());
    let rev: Vec<i32> = a.into_iter().rev().collect();
    println!("{:?}", rev);
    for (i, x) in a.iter().rev().enumerate() {
        print!("{}:{} ", i, x);
    }
    println!();

    let mut tiles = [['.'; 4]; 3];
    tiles[1][2] = '#';
    tiles[0] = ['a', 'b', 'c', 'd'];
    let rows: Vec<String> = tiles.iter().map(|r| r.iter().collect()).collect();
    println!("{}", rows.join("/"));
    println!("{:?}", tiles[2]);
    println!("{}", tiles.iter().flatten().filter(|&&c| c == '.').count());
    println!("{:?}", tiles.iter().position(|r| r.contains(&'#')));
    let biggest = [[3, 9], [7, 1], [4, 4]].iter().copied().max_by_key(|p| p[0] + p[1]);
    println!("{:?}", biggest);
    let nested = [[[0u8; 2]; 2]; 2];
    println!("{:?} {}", nested, std::mem::size_of_val(&nested));
}
